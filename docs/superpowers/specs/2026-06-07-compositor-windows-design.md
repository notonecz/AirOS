# Compositor Window Rendering + airdemo Design (Sub-projekt 3a)

**Goal:** Zobrazit app window obsahující checkerboard pixel buffer v aircomp okně, složený pod shell chrome (topbar, dock).

**Scope:** MVP bez dekorací oken, bez resize, bez vstupu — jen textura z IPC bufferu vykreslená na obrazovce. App interakce přijde v pozdějším plánu.

---

## Architektura

```
airdemo (apps/airdemo)                aircomp (crates/aircomp)
  └── airproto client                   └── surface.rs (render loop)
       ├── WindowCreate(id=1, 640×480)       ├── TexturePipeline (NOVÉ)
       ├── BufferCommit(id=1, pixels)        │    Vec<u8> RGBA → wgpu::Texture → quad
       └── recv WindowClose → exit           └── render_frame():
                                                  1. clear #141414 (LoadOp::Clear)
                                                  2. TexturePipeline::draw(windows) ← NOVÉ
                                                  3. RectPipeline::draw(shell_chrome)
```

Frame render pořadí (back to front): desktop clear → app window textury → shell chrome rects.

---

## Změny WindowState — přidání pozice

`crates/aircomp/src/window.rs` — přidat `position: Point`:

```rust
pub struct WindowState {
    pub size: Size,
    pub position: Point,   // ← NOVÉ (pixel coords, origin = levý horní roh)
    pub buffer: Vec<u8>,
}
```

`crates/aircomp/src/scene.rs` — `add_window` přijme také `position: Point`. Výchozí pozice se vypočítá v `surface.rs` při `WindowCreate` jako střed obrazovky.

`airproto/src/messages.rs` — `ClientMessage::WindowCreate` se nemění (position určuje compositor, ne klient). Compositor centruje okno na základě `screen_w/h - window_w/h`.

---

## Soubory

### Nové soubory

**`crates/aircomp/src/texture_pipeline.rs`**
- `TexturePipeline::new(device, format) -> Self`
- `TexturePipeline::draw(encoder, view, queue, windows: &[WindowDraw])` — pro každé okno: upload RGBA pixelů do `wgpu::Texture`, vytvoří bind group, vykreslí quad (6 vertexů, stejný NDC přístup jako RectPipeline)
- `struct WindowDraw { x: f32, y: f32, w: f32, h: f32, pixels: &[u8] }` — pixel souřadnice (0..screen_w, 0..screen_h)
- WGSL vertex shader: pixel rect → NDC (stejný uniform `ScreenSize` jako RectPipeline)
- WGSL fragment shader: `textureSample(t_window, s_window, uv)` — passthrough z textury

**`apps/airdemo/Cargo.toml`**
```toml
[package]
name = "airdemo"
version.workspace = true
edition.workspace = true

[[bin]]
name = "airdemo"
path = "src/main.rs"

[dependencies]
airproto = { path = "../../crates/airproto" }
tokio = { workspace = true }
```

**`apps/airdemo/src/main.rs`**
1. Připojí se k `/tmp/aircomp.sock`
2. Odešle `WindowCreate { id: WindowId(1), size: 640×480 }`
3. Vygeneruje 640×480 checkerboard: 8×8px čtverce, střídají se `[0,0,0,255]` (černá) a `[255,255,255,255]` (bílá) v RGBA formátu
4. Odešle `BufferCommit { id: WindowId(1), data }`
5. Čeká na `ServerMessage::WindowClose { window_id: WindowId(1) }` → exit

### Změněné soubory

**`crates/aircomp/src/window.rs`**
- Přidat `position: Point` do `WindowState`
- `WindowState::new(size, position) -> Self` — přijme pozici jako parametr

**`crates/aircomp/src/scene.rs`**
- `add_window(id, size, position: Point)` — předá pozici do `WindowState::new`
- Stávající `resize_window` a `commit_buffer` bez změny

**`crates/aircomp/src/ipc_server.rs`**
- `WindowCreate` handler: vypočítat výchozí pozici jako `(screen_w/2 - size.w/2, screen_h/2 - size.h/2)` — ale screen_w/h není v IPC serveru dostupná
- **Řešení:** použít fixní výchozí pozici `Point { x: 100, y: 100 }` (screen size není v IPC kontextu dostupná — centrování se přidá v pozdějším plánu s notifikačním kanálem)

**`crates/aircomp/src/lib.rs`**
- `pub mod texture_pipeline;`

**`crates/aircomp/src/surface.rs`**
- `App` struct přidá `texture_pipeline: Option<TexturePipeline>`
- `SurfaceRenderer::render_frame` rozšíření: po clear pasu a před shell chrome — iteruj `scene.z_order()`, pro každé okno s neprázdným bufferem zavolej `texture_pipeline.draw(...)`

**`Cargo.toml` (workspace)**
- Přidat `"apps/airdemo"` do `members`

---

## TexturePipeline — detail

### WGSL shadery

```wgsl
// Vertex — stejný přístup jako RectPipeline (NDC z pixelů)
struct ScreenSize { width: f32, height: f32 }
@group(0) @binding(0) var<uniform> screen: ScreenSize;

struct VertexOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> }

@vertex fn vs_main(@builtin(vertex_index) vi: u32,
                   @builtin(instance_index) ii: u32) -> VertexOut {
    // corner lookup: [(0,0),(1,0),(0,1),(1,0),(1,1),(0,1)]
    // window rect z uniform bufferu (x,y,w,h)
    // NDC: ndcx = px/screen.w * 2.0 - 1.0, ndcy = 1.0 - py/screen.h * 2.0
}

@group(1) @binding(0) var t_window: texture_2d<f32>;
@group(1) @binding(1) var s_window: sampler;

@fragment fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(t_window, s_window, in.uv);
}
```

Každé okno dostane vlastní `wgpu::Texture` (vytvořená per-frame, rozměr = `window.size`). Buffer se uploaduje přes `queue.write_texture`. Bind group pro texturu se vytváří per-draw.

### Render pořadí v surface.rs

```rust
// 1. clear pass (#141414)
// 2. texture pass — app windows
for id in scene.z_order() {
    let win = scene.get(*id).unwrap();
    if !win.buffer.is_empty() {
        texture_pipeline.draw(encoder, view, queue, &[WindowDraw {
            x: win.position.x as f32,
            y: win.position.y as f32,
            w: win.size.width as f32,
            h: win.size.height as f32,
            pixels: &win.buffer,
        }]);
    }
}
// 3. rect pass — shell chrome
rect_pipeline.draw(encoder, view, queue, &shell_chrome_rects);
```

---

## Testování

### Unit testy (bez GPU)

`texture_pipeline.rs`:
```rust
#[test]
fn texture_pipeline_exists_as_type() {
    // stejný vzor jako existující test v surface.rs
}
```

`window.rs`:
```rust
#[test]
fn window_state_has_position() {
    let state = WindowState::new(
        Size { width: 640, height: 480 },
        Point { x: 100, y: 100 }
    );
    assert_eq!(state.position.x, 100);
}
```

### Manuální test

```bash
# Terminal 1
cargo run -p aircomp
# Očekávaný výsledek: AirOS okno se shell chrome (topbar + dock)

# Terminal 2
cargo run -p airdemo
# Očekávaný výsledek: checkerboard okno se objeví v aircomp,
#                     vlevo nahoře (pozice 100,100),
#                     pod shell chrome.
# Zavřít airdemo → okno zmizí z aircomp.
```

---

## Co je mimo scope tohoto sub-projektu

- Dekorace oken (title bar, close/minimize/maximize tlačítka) — Sub-projekt 3b+
- Window drag/resize input handling
- Centrování okna na základě skutečné velikosti obrazovky
- airterm (Sub-projekt 3b)
- Více oken současně (funguje díky z-order, ale není testováno)

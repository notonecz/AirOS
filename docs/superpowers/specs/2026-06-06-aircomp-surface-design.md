# AirComp Surface Rendering Design

**Goal:** Zobrazit AirOS shell chrome (topbar, floating dock, tmavé pozadí) v reálném OS okně pomocí winit + wgpu Surface.

**Scope:** MVP bez reálných appek — jen shell chrome. App okna přijdou v dalším plánu.

---

## Architektura

```
main.rs → aircomp::run()
              ├── tokio task: ipc_server  (beze změny)
              └── winit EventLoop (hlavní vlákno)
                      ├── WindowEvent::RedrawRequested → render_frame()
                      └── WindowEvent::CloseRequested → exit
```

Winit event loop musí běžet na hlavním vlákně (požadavek macOS). IPC server běží v tokio tasku na pozadí.

---

## Soubory

### Nové soubory

**`crates/aircomp/src/surface.rs`**
- Vytvoří `winit::Window` a inicializuje `wgpu::Surface` na něj.
- Poskytuje `SurfaceRenderer { device, queue, surface, config }`.
- `SurfaceRenderer::new(window) -> Option<Self>` — vrátí None bez GPU (CI).
- `SurfaceRenderer::render_frame(rects: &[ColoredRect])` — clear + rect pass + present.
- `SurfaceRenderer::resize(new_size)` — překonfiguruje Surface při změně velikosti okna.

**`crates/aircomp/src/rect_pipeline.rs`**
- WGSL shader: vertex vstup = `[x, y, w, h]` + `[r, g, b, a]` → 2 trojúhelníky.
- `RectPipeline::new(device, format) -> Self`
- `RectPipeline::draw(encoder, view, rects: &[ColoredRect])` — nahraje vertex buffer, zavolá draw call.
- `ColoredRect { x: f32, y: f32, w: f32, h: f32, color: [f32; 4] }` — normalizované souřadnice (0..1 od levého horního rohu).

**`crates/aircomp/src/shell_chrome.rs`**
- Čistý Rust, žádný GPU kód.
- `compute_chrome(screen_w: u32, screen_h: u32) -> Vec<ColoredRect>`
- Vrátí:
  1. **Desktop fill** — celá plocha, `#141414` (rgba 0.078, 0.078, 0.078, 1.0)
  2. **Topbar** — `x:0, y:0, w:screen_w, h:28px`, barva `#1C1C1E`
  3. **Dock bg** — zaoblený rect 232×56px, dole uprostřed + 8px margin, `rgba(44,44,46,0.85)` (průhlednost aproximovaná — wgpu alpha blending)
  4. **Dock placeholder ikony** — 4× rect 42×42px v dock bg

### Změněné soubory

**`crates/aircomp/Cargo.toml`**
- Přidat: `winit = "0.30"` (kompatibilní s wgpu 22 / macOS Metal)

**`crates/aircomp/src/lib.rs`**
- `pub mod surface; pub mod rect_pipeline; pub mod shell_chrome;`
- `pub async fn run()` — přepsat: spustí IPC task, pak předá kontrolu winit event loopu.
- Zachovat `run_headless()` v `renderer.rs` pro CI testy.

**`crates/aircomp/src/main.rs`**
- Beze změny (`aircomp::run().await`).

---

## Render pipeline — detail

### ColoredRect souřadnicový systém

`shell_chrome::compute_chrome()` vrací recty v **pixelech** (origin = levý horní roh). `RectPipeline::draw()` je převede na NDC (Normalized Device Coordinates) pomocí uniform bufferu s rozlišením.

### WGSL vertex shader (pseudokód)

```wgsl
struct Rect { x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32 }

@vertex fn vs_main(@builtin(vertex_index) vi: u32,
                   @builtin(instance_index) ii: u32) -> ... {
    // vi 0..5: 2 trojúhelníky z jednoho rectu
    let rect = rects[ii];
    let corner = corners[vi];  // [(0,0),(1,0),(0,1),(1,0),(1,1),(0,1)]
    let px = rect.x + corner.x * rect.w;
    let py = rect.y + corner.y * rect.h;
    // pixel → NDC
    let ndcx = px / screen_size.x * 2.0 - 1.0;
    let ndcy = 1.0 - py / screen_size.y * 2.0;
    return vec4<f32>(ndcx, ndcy, 0.0, 1.0);
}

@fragment fn fs_main(...) -> @location(0) vec4<f32> {
    return color;  // passthrough z vertex
}
```

### Frame sekvence

1. `compute_chrome(w, h)` → `Vec<ColoredRect>`
2. `SurfaceRenderer::render_frame(rects)`:
   - `surface.get_current_texture()` → frame
   - clear pass: `LoadOp::Clear` s `#141414`
   - rect pass: `RectPipeline::draw(rects)`
   - `queue.submit()` + `frame.present()`

---

## Testování

### Unit testy (bez GPU)

`shell_chrome.rs`:
```rust
#[test]
fn compute_chrome_returns_correct_rect_count() {
    let rects = compute_chrome(1920, 1080);
    assert_eq!(rects.len(), 7); // fill + topbar + dock_bg + 4 ikony
}

#[test]
fn topbar_rect_spans_full_width() {
    let rects = compute_chrome(1920, 1080);
    let topbar = &rects[1];
    assert_eq!(topbar.x, 0.0);
    assert_eq!(topbar.w, 1920.0);
    assert_eq!(topbar.h, 28.0);
}

#[test]
fn dock_bg_is_centered() {
    let rects = compute_chrome(1920, 1080);
    let dock = &rects[2];
    assert!((dock.x - (1920.0 - 232.0) / 2.0).abs() < 1.0);
}
```

`rect_pipeline.rs` + `surface.rs`:
```rust
#[tokio::test]
async fn surface_renderer_initializes_or_skips() {
    // Stejný vzor jako existující AirCompRenderer test
}
```

### Manuální test

```bash
cargo run -p aircomp
# Očekávaný výsledek: OS okno s tmavou plochou, topbarem nahoře, dockem dole
```

---

## Co je mimo scope

- Reálné ikony v docku (emoji placeholder)
- Text rendering v topbaru (jen barevný pruh)
- App okna (příští plán)
- Napojení `AirShell` state na rendering (příští plán)
- Průhlednost docku přes backdrop blur (aproximace pevnou barvou)

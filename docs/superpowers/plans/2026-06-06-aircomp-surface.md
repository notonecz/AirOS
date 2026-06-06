# AirComp Surface Rendering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Zobrazit AirOS shell chrome (topbar 28px + floating dock dole) v reálném OS okně pomocí winit + wgpu Surface na macOS.

**Architecture:** `main.rs` zůstane triviální, ale `run()` přestane být `async fn` — místo toho vytvoří tokio runtime, spustí IPC server jako task, a předá kontrolu winit event loopu na hlavním vlákně. Shell chrome se vykreslí přes wgpu RenderPipeline se WGSL shaderem, který přijímá barevné obdélníky jako instance data.

**Tech Stack:** Rust, wgpu 22 (Metal backend na macOS), winit 0.30 (ApplicationHandler API), pollster 0.3 (sync block_on pro async wgpu init z winit callback), tokio (IPC server).

---

## Soubory

```
crates/aircomp/
├── Cargo.toml                 MODIFY — add winit + pollster
└── src/
    ├── main.rs                MODIFY — change to fn main() (non-async)
    ├── lib.rs                 MODIFY — add modules, change run() to sync
    ├── shell_chrome.rs        CREATE — ColoredRect + compute_chrome()
    ├── rect_pipeline.rs       CREATE — WGSL shader + RectPipeline
    ├── surface.rs             CREATE — SurfaceRenderer + winit App + run_event_loop()
    ├── renderer.rs            NO CHANGE — headless renderer pro CI testy
    ├── scene.rs               NO CHANGE
    ├── window.rs              NO CHANGE
    └── ipc_server.rs          NO CHANGE
```

---

## Task 1: Crate setup — deps + stubs + main.rs

**Files:**
- Modify: `crates/aircomp/Cargo.toml`
- Modify: `crates/aircomp/src/main.rs`
- Create stubs: `crates/aircomp/src/shell_chrome.rs`, `crates/aircomp/src/rect_pipeline.rs`, `crates/aircomp/src/surface.rs`
- Modify: `crates/aircomp/src/lib.rs`

- [ ] **Step 1: Přidej deps do `crates/aircomp/Cargo.toml`**

```toml
[package]
name = "aircomp"
version.workspace = true
edition.workspace = true

[[bin]]
name = "aircomp"
path = "src/main.rs"

[lib]
name = "aircomp"
path = "src/lib.rs"

[dependencies]
airproto = { path = "../airproto" }
tokio = { workspace = true }
wgpu = { workspace = true }
winit = "0.30"
pollster = "0.3"

[dev-dependencies]
tempfile = { workspace = true }
```

- [ ] **Step 2: Přepiš `crates/aircomp/src/main.rs` na non-async**

```rust
fn main() {
    aircomp::run();
}
```

- [ ] **Step 3: Vytvoř stub `crates/aircomp/src/shell_chrome.rs`**

```rust
// crates/aircomp/src/shell_chrome.rs
```

- [ ] **Step 4: Vytvoř stub `crates/aircomp/src/rect_pipeline.rs`**

```rust
// crates/aircomp/src/rect_pipeline.rs
```

- [ ] **Step 5: Vytvoř stub `crates/aircomp/src/surface.rs`**

```rust
// crates/aircomp/src/surface.rs
```

- [ ] **Step 6: Aktualizuj `crates/aircomp/src/lib.rs` — přidej moduly a změň run() na sync**

```rust
pub mod ipc_server;
pub mod rect_pipeline;
pub mod renderer;
pub mod scene;
pub mod shell_chrome;
pub mod surface;
pub mod window;

use std::sync::{Arc, Mutex};

pub use rect_pipeline::ColoredRect;
pub use scene::WindowScene;
pub use window::WindowState;

pub fn run() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let scene = Arc::new(Mutex::new(WindowScene::new()));
    let socket_path = std::path::PathBuf::from("/tmp/aircomp.sock");
    let _ = std::fs::remove_file(&socket_path);

    let ipc_scene = scene.clone();
    rt.spawn(async move {
        ipc_server::run_server(ipc_scene, &socket_path)
            .await
            .expect("IPC server failed");
    });

    surface::run_event_loop(scene);
}

pub async fn run_headless(scene: Arc<Mutex<WindowScene>>) {
    renderer::run_headless(scene).await;
}
```

- [ ] **Step 7: Ověř build**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo check -p aircomp 2>&1 | tail -5
```

Očekávaný výstup: `Finished` (warnings o prázdných modulech jsou OK)

- [ ] **Step 8: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/aircomp/
git commit -m "chore(aircomp): add winit+pollster deps, switch run() to sync, add module stubs"
```

---

## Task 2: shell_chrome.rs — ColoredRect + compute_chrome()

**Files:**
- Modify: `crates/aircomp/src/shell_chrome.rs`

Tato vrstva je čistý Rust bez GPU — produkuje seznam barevných obdélníků, které SurfaceRenderer vykreslí. `ColoredRect` je definován v `rect_pipeline.rs`; `shell_chrome.rs` ho importuje. Desktop fill řeší `LoadOp::Clear` v render passovi, takže `compute_chrome` vrací 6 rektů: topbar + dock bg + 4 ikony.

- [ ] **Step 1: Napiš testy**

```rust
// crates/aircomp/src/shell_chrome.rs — začni testy

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_chrome_returns_six_rects() {
        let rects = compute_chrome(1920, 1080);
        assert_eq!(rects.len(), 6);
    }

    #[test]
    fn topbar_spans_full_width() {
        let rects = compute_chrome(1920, 1080);
        let topbar = &rects[0];
        assert_eq!(topbar.x, 0.0);
        assert_eq!(topbar.y, 0.0);
        assert_eq!(topbar.w, 1920.0);
        assert_eq!(topbar.h, 28.0);
    }

    #[test]
    fn dock_bg_is_horizontally_centered() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        let expected_x = (1920.0 - 232.0) / 2.0;
        assert!((dock.x - expected_x).abs() < 0.5);
        assert_eq!(dock.w, 232.0);
        assert_eq!(dock.h, 56.0);
    }

    #[test]
    fn dock_bg_is_near_bottom() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        let expected_y = 1080.0 - 56.0 - 8.0;
        assert!((dock.y - expected_y).abs() < 0.5);
    }

    #[test]
    fn four_icon_rects_are_within_dock_bg() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        for icon in &rects[2..6] {
            assert!(icon.x >= dock.x);
            assert!(icon.x + icon.w <= dock.x + dock.w + 0.5);
            assert!(icon.y >= dock.y);
            assert_eq!(icon.w, 42.0);
            assert_eq!(icon.h, 42.0);
        }
    }

    #[test]
    fn colored_rect_fields_accessible() {
        use crate::rect_pipeline::ColoredRect;
        let r = ColoredRect { x: 1.0, y: 2.0, w: 3.0, h: 4.0, color: [1.0, 0.0, 0.0, 1.0] };
        assert_eq!(r.x, 1.0);
        assert_eq!(r.color[0], 1.0);
    }
}
```

- [ ] **Step 2: Spusť testy — musí selhat**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p aircomp shell_chrome 2>&1 | tail -5
```

Očekávaný výstup: FAIL (compute_chrome undefined)

- [ ] **Step 3: Napiš implementaci**

```rust
// crates/aircomp/src/shell_chrome.rs

use crate::rect_pipeline::ColoredRect;

// Shell chrome barvy (macOS-inspired dark palette)
const COLOR_TOPBAR: [f32; 4] = [0.110, 0.110, 0.118, 1.0]; // #1C1C1E
const COLOR_DOCK_BG: [f32; 4] = [0.173, 0.173, 0.180, 0.92]; // #2C2C2E @92%
const COLOR_DOCK_ICON: [f32; 4] = [0.220, 0.220, 0.227, 1.0]; // #383838

const TOPBAR_H: f32 = 28.0;
const DOCK_W: f32 = 232.0;
const DOCK_H: f32 = 56.0;
const DOCK_MARGIN_BOTTOM: f32 = 8.0;
const DOCK_ICON_SIZE: f32 = 42.0;
const DOCK_ICON_GAP: f32 = 8.0;
// side padding = (DOCK_W - 4*DOCK_ICON_SIZE - 3*DOCK_ICON_GAP) / 2 = (232 - 168 - 24) / 2 = 20
const DOCK_ICON_SIDE_PAD: f32 = 20.0;

/// Vypočítá seznam obdélníků pro shell chrome.
///
/// Pořadí: topbar, dock bg, 4× dock icon placeholder (celkem 6).
/// Desktop fill (pozadí #141414) je řešen pomocí clear passus v `SurfaceRenderer::render_frame`.
pub fn compute_chrome(screen_w: u32, screen_h: u32) -> Vec<ColoredRect> {
    let sw = screen_w as f32;
    let sh = screen_h as f32;

    let dock_x = (sw - DOCK_W) / 2.0;
    let dock_y = sh - DOCK_H - DOCK_MARGIN_BOTTOM;
    let icon_y = dock_y + (DOCK_H - DOCK_ICON_SIZE) / 2.0;

    let mut rects = vec![
        // 0: topbar
        ColoredRect { x: 0.0, y: 0.0, w: sw, h: TOPBAR_H, color: COLOR_TOPBAR },
        // 1: dock bg
        ColoredRect { x: dock_x, y: dock_y, w: DOCK_W, h: DOCK_H, color: COLOR_DOCK_BG },
    ];

    // 2–5: dock icon placeholders
    for i in 0..4u32 {
        let icon_x = dock_x + DOCK_ICON_SIDE_PAD + i as f32 * (DOCK_ICON_SIZE + DOCK_ICON_GAP);
        rects.push(ColoredRect {
            x: icon_x,
            y: icon_y,
            w: DOCK_ICON_SIZE,
            h: DOCK_ICON_SIZE,
            color: COLOR_DOCK_ICON,
        });
    }

    rects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_chrome_returns_six_rects() {
        let rects = compute_chrome(1920, 1080);
        assert_eq!(rects.len(), 6);
    }

    #[test]
    fn topbar_spans_full_width() {
        let rects = compute_chrome(1920, 1080);
        let topbar = &rects[0];
        assert_eq!(topbar.x, 0.0);
        assert_eq!(topbar.y, 0.0);
        assert_eq!(topbar.w, 1920.0);
        assert_eq!(topbar.h, 28.0);
    }

    #[test]
    fn dock_bg_is_horizontally_centered() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        let expected_x = (1920.0 - 232.0) / 2.0;
        assert!((dock.x - expected_x).abs() < 0.5);
        assert_eq!(dock.w, 232.0);
        assert_eq!(dock.h, 56.0);
    }

    #[test]
    fn dock_bg_is_near_bottom() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        let expected_y = 1080.0 - 56.0 - 8.0;
        assert!((dock.y - expected_y).abs() < 0.5);
    }

    #[test]
    fn four_icon_rects_are_within_dock_bg() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        for icon in &rects[2..6] {
            assert!(icon.x >= dock.x);
            assert!(icon.x + icon.w <= dock.x + dock.w + 0.5);
            assert!(icon.y >= dock.y);
            assert_eq!(icon.w, 42.0);
            assert_eq!(icon.h, 42.0);
        }
    }

    #[test]
    fn colored_rect_fields_accessible() {
        let r = ColoredRect { x: 1.0, y: 2.0, w: 3.0, h: 4.0, color: [1.0, 0.0, 0.0, 1.0] };
        assert_eq!(r.x, 1.0);
        assert_eq!(r.color[0], 1.0);
    }
}
```

- [ ] **Step 4: Spusť testy — musí projít**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p aircomp shell_chrome 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 6 passed`

- [ ] **Step 5: Cargo fmt**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo fmt -p aircomp
```

- [ ] **Step 6: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/aircomp/src/shell_chrome.rs
git commit -m "feat(aircomp): add ColoredRect and compute_chrome shell chrome layout"
```

---

## Task 3: rect_pipeline.rs — WGSL shader + RectPipeline

**Files:**
- Modify: `crates/aircomp/src/rect_pipeline.rs`

Pipeline přijme seznam `ColoredRect`, nahraje je jako instance data do GPU bufferu, a vykreslí je přes 2 trojúhelníky na instanci (6 vertex_index hodnot). Uniform buffer drží rozlišení obrazovky pro převod pixelů → NDC.

- [ ] **Step 1: Napiš test (GPU-optional)**

```rust
// crates/aircomp/src/rect_pipeline.rs — začni testem

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_device() -> Option<wgpu::Device> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        let (device, _queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .ok()?;
        Some(device)
    }

    #[tokio::test]
    async fn rect_pipeline_creates_without_panic() {
        let Some(device) = make_device().await else {
            return; // Žádný GPU adapter — OK v CI
        };
        let _pipeline = RectPipeline::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    }
}
```

- [ ] **Step 2: Spusť test — musí selhat**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p aircomp rect_pipeline 2>&1 | tail -5
```

Očekávaný výstup: FAIL (RectPipeline undefined)

- [ ] **Step 3: Napiš implementaci**

```rust
// crates/aircomp/src/rect_pipeline.rs

use crate::shell_chrome::ColoredRect;

const SHADER_SRC: &str = r#"
struct ScreenSize {
    width: f32,
    height: f32,
};

@group(0) @binding(0) var<uniform> screen: ScreenSize;

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let c = corners[vi];
    let px = rect.x + c.x * rect.z;
    let py = rect.y + c.y * rect.w;
    let ndcx = px / screen.width * 2.0 - 1.0;
    let ndcy = 1.0 - py / screen.height * 2.0;
    var out: VertexOut;
    out.pos = vec4<f32>(ndcx, ndcy, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
"#;

const MAX_RECTS: u64 = 64;

pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    screen_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl RectPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rect_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let screen_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screen_size"),
            size: 8, // 2 × f32
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect_instances"),
            size: MAX_RECTS * 32, // každý rect = 8 × f32 = 32 bajtů
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rect_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rect_bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rect_layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            vertex_buf,
            screen_buf,
            bind_group,
        }
    }

    /// Aktualizuje uniform buffer s rozlišením obrazovky.
    /// Volat po každé změně velikosti okna a po inicializaci.
    pub fn update_screen_size(&self, queue: &wgpu::Queue, w: f32, h: f32) {
        let data: [f32; 2] = [w, h];
        let bytes = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, 8)
        };
        queue.write_buffer(&self.screen_buf, 0, bytes);
    }

    /// Nahraje recty do GPU a vykreslí je do `view`.
    /// Volá se uvnitř existujícího command encoderu (po clear passovi).
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        rects: &[ColoredRect],
    ) {
        if rects.is_empty() {
            return;
        }
        assert!(
            rects.len() <= MAX_RECTS as usize,
            "Too many rects: {}",
            rects.len()
        );

        // Zabal instance data do bajtů: každý rect = [x, y, w, h, r, g, b, a]
        let mut data: Vec<f32> = Vec::with_capacity(rects.len() * 8);
        for r in rects {
            data.extend_from_slice(&[r.x, r.y, r.w, r.h]);
            data.extend_from_slice(&r.color);
        }
        let bytes = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<f32>(),
            )
        };
        queue.write_buffer(&self.vertex_buf, 0, bytes);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("rect_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.draw(0..6, 0..rects.len() as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_device() -> Option<wgpu::Device> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        let (device, _queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .ok()?;
        Some(device)
    }

    #[tokio::test]
    async fn rect_pipeline_creates_without_panic() {
        let Some(device) = make_device().await else {
            return; // Žádný GPU adapter — OK v CI
        };
        let _pipeline = RectPipeline::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    }
}
```

**Poznámka k WGSL:** V shaderu `rect.z` a `rect.w` jsou třetí a čtvrtá složka vec4 — tedy šířka a výška rectu (protože `@location(0) rect: vec4<f32>` přijímá `[x, y, w, h]`).

- [ ] **Step 4: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p aircomp rect_pipeline 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 1 passed` (nebo `0 passed` bez GPU — obojí je OK)

- [ ] **Step 5: Cargo fmt**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo fmt -p aircomp
```

- [ ] **Step 6: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/aircomp/src/rect_pipeline.rs
git commit -m "feat(aircomp): add RectPipeline with WGSL instanced rect shader"
```

---

## Task 4: surface.rs — SurfaceRenderer + winit App

**Files:**
- Modify: `crates/aircomp/src/surface.rs`

`SurfaceRenderer` obaluje wgpu Surface (reálné okno místo offscreen textury). `App` implementuje winit 0.30 `ApplicationHandler` trait. `run_event_loop()` je veřejná funkce která spustí winit event loop.

- [ ] **Step 1: Napiš test**

```rust
// crates/aircomp/src/surface.rs — začni testem

#[cfg(test)]
mod tests {
    #[test]
    fn run_event_loop_exists_as_function() {
        // Kompilační test — ověří že funkce existuje se správnou signaturou.
        // Nelze spustit bez okna (vyžaduje display server).
        let _: fn(std::sync::Arc<std::sync::Mutex<crate::scene::WindowScene>>) =
            super::run_event_loop;
    }
}
```

- [ ] **Step 2: Spusť test — musí selhat**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p aircomp surface 2>&1 | tail -5
```

Očekávaný výstup: FAIL (run_event_loop undefined)

- [ ] **Step 3: Napiš implementaci**

```rust
// crates/aircomp/src/surface.rs

use crate::rect_pipeline::RectPipeline;
use crate::scene::WindowScene;
use crate::shell_chrome::{self, ColoredRect};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

pub struct SurfaceRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: RectPipeline,
}

impl SurfaceRenderer {
    /// Inicializuje wgpu Surface na daném okně.
    /// Vrátí None pokud není dostupný GPU adapter (CI bez GPU).
    pub async fn new(window: Arc<Window>) -> Option<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).ok()?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .ok()?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let pipeline = RectPipeline::new(&device, format);
        pipeline.update_screen_size(&queue, size.width as f32, size.height as f32);

        Some(Self {
            device,
            queue,
            surface,
            config,
            pipeline,
        })
    }

    /// Překonfiguruje Surface po změně velikosti okna.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.pipeline
            .update_screen_size(&self.queue, width as f32, height as f32);
    }

    /// Vykreslí jeden frame: clear na #141414, pak recty.
    pub fn render_frame(&self, rects: &[ColoredRect]) {
        let Ok(frame) = self.surface.get_current_texture() else {
            return;
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("frame_encoder"),
                });

        // Clear pass — desktop pozadí #141414
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.078,
                            g: 0.078,
                            b: 0.078,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.pipeline
            .draw(&mut encoder, &view, &self.queue, rects);
        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}

struct App {
    scene: Arc<Mutex<WindowScene>>,
    window: Option<Arc<Window>>,
    renderer: Option<SurfaceRenderer>,
}

impl App {
    fn new(scene: Arc<Mutex<WindowScene>>) -> Self {
        Self {
            scene,
            window: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("AirOS")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 800u32)),
                )
                .expect("Failed to create window"),
        );
        let renderer = pollster::block_on(SurfaceRenderer::new(window.clone()));
        self.window = Some(window);
        self.renderer = renderer;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size.width, size.height);
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(window)) = (&self.renderer, &self.window) {
                    let size = window.inner_size();
                    let rects =
                        shell_chrome::compute_chrome(size.width, size.height);
                    renderer.render_frame(&rects);
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Spustí winit event loop — blokuje dokud uživatel nezavře okno.
/// Musí být voláno z hlavního vlákna.
pub fn run_event_loop(scene: Arc<Mutex<WindowScene>>) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new(scene);
    event_loop
        .run_app(&mut app)
        .expect("Event loop failed");
}

#[cfg(test)]
mod tests {
    #[test]
    fn run_event_loop_exists_as_function() {
        let _: fn(std::sync::Arc<std::sync::Mutex<crate::scene::WindowScene>>) =
            super::run_event_loop;
    }
}
```

- [ ] **Step 4: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p aircomp surface 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 1 passed`

- [ ] **Step 5: Cargo fmt**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo fmt -p aircomp
```

- [ ] **Step 6: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/aircomp/src/surface.rs
git commit -m "feat(aircomp): add SurfaceRenderer and winit ApplicationHandler"
```

---

## Task 5: Finální drátování + manuální test

**Files:**
- Modify: `crates/aircomp/src/lib.rs` (přidej re-export `ColoredRect`)

V Task 1 jsme už napsali finální `lib.rs`. Tento task ověří že vše kompiluje, testy projdou, a spustí binárku.

- [ ] **Step 1: Spusť všechny aircomp testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p aircomp 2>&1 | tail -10
```

Očekávaný výstup: `test result: ok. N passed` — musí projít všechny existující testy plus nové (shell_chrome: 6, rect_pipeline: 1, surface: 1).

- [ ] **Step 2: Clippy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo clippy -p aircomp -- -D warnings 2>&1 | tail -5
```

Očekávaný výstup: `Finished` bez warningů.

- [ ] **Step 3: cargo fmt --check**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo fmt --check -p aircomp 2>&1 | tail -3
```

Očekávaný výstup: žádný výstup (= formátování OK)

- [ ] **Step 4: Manuální spuštění**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo run -p aircomp
```

Očekávaný výsledek: otevře se OS okno 1280×800 s tmavou plochou (#141414), tmavším topbarem nahoře (#1C1C1E, 28px) a floating dockem dole uprostřed se 4 šedými ikonami.

Zavři okno křížkem nebo Cmd+W. Program se ukončí.

- [ ] **Step 5: Commit (pokud byly nutné opravy)**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/aircomp/src/
git commit -m "fix(aircomp): final wiring and clippy fixes"
```

(Pokud nebyly žádné opravy, commit přeskoč.)

---

## Spec coverage check

| Spec požadavek | Task |
|---|---|
| `winit` dep přidán | Task 1 |
| `surface.rs` — `SurfaceRenderer::new(window)` | Task 4 |
| `surface.rs` — `resize(w, h)` | Task 4 |
| `surface.rs` — `render_frame(rects)` | Task 4 |
| `surface.rs` — `run_event_loop(scene)` | Task 4 |
| `rect_pipeline.rs` — `ColoredRect` | Task 3 |
| `rect_pipeline.rs` — `RectPipeline::new(device, format)` | Task 3 |
| `rect_pipeline.rs` — `update_screen_size(queue, w, h)` | Task 3 |
| `rect_pipeline.rs` — `draw(encoder, view, queue, rects)` | Task 3 |
| WGSL vertex shader (pixel → NDC přes uniform) | Task 3 |
| `shell_chrome.rs` — `compute_chrome(w, h) -> Vec<ColoredRect>` | Task 2 |
| Topbar 28px nahoře | Task 2 |
| Dock bg dole uprostřed 232×56px + 8px margin | Task 2 |
| 4 dock icon placeholders | Task 2 |
| `lib.rs::run()` — sync, vytvoří tokio runtime | Task 1 |
| `lib.rs::run()` — IPC server jako tokio task | Task 1 |
| `lib.rs::run()` — volá `run_event_loop` | Task 1 |
| `main.rs` — `fn main()` non-async | Task 1 |
| Clear pass s `#141414` | Task 4 |
| Testy pro `shell_chrome` (6 testů) | Task 2 |
| Testy pro `rect_pipeline` (GPU-optional) | Task 3 |
| Manuální test `cargo run -p aircomp` | Task 5 |

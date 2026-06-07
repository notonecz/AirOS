// crates/aircomp/src/surface.rs

use crate::rect_pipeline::RectPipeline;
use crate::scene::WindowScene;
use crate::shell_chrome;
use crate::texture_pipeline::{TexturePipeline, WindowDraw};
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
    texture_pipeline: TexturePipeline,
}

impl SurfaceRenderer {
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

        let texture_pipeline = TexturePipeline::new(&device, format);
        texture_pipeline.update_screen_size(&queue, size.width as f32, size.height as f32);

        Some(Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            texture_pipeline,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.pipeline
            .update_screen_size(&self.queue, width as f32, height as f32);
        self.texture_pipeline
            .update_screen_size(&self.queue, width as f32, height as f32);
    }

    /// Vykreslí jeden frame:
    /// 1. clear na #141414
    /// 2. app window textury (z WindowScene, v z-pořadí)
    /// 3. shell chrome rects (topbar, dock)
    pub fn render_frame(
        &self,
        rects: &[crate::rect_pipeline::ColoredRect],
        scene: &WindowScene,
    ) {
        let Ok(frame) = self.surface.get_current_texture() else {
            return;
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        // 1. Clear pass — desktop pozadí #141414
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

        // 2. App window textury (back-to-front z-order)
        for &id in scene.z_order() {
            if let Some(win) = scene.get(id) {
                if !win.buffer.is_empty() {
                    let draw = WindowDraw {
                        x: win.position.x,
                        y: win.position.y,
                        w: win.size.width as f32,
                        h: win.size.height as f32,
                        pixels: &win.buffer,
                    };
                    self.texture_pipeline.draw(
                        &mut encoder,
                        &view,
                        &self.device,
                        &self.queue,
                        &draw,
                    );
                }
            }
        }

        // 3. Shell chrome (topbar, dock) — vždy nahoře
        self.pipeline.draw(&mut encoder, &view, &self.queue, rects);

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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
                    let rects = shell_chrome::compute_chrome(size.width, size.height);
                    let scene = self.scene.lock().unwrap();
                    renderer.render_frame(&rects, &scene);
                    drop(scene);
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
    event_loop.run_app(&mut app).expect("Event loop failed");
}

#[cfg(test)]
mod tests {
    #[test]
    fn run_event_loop_exists_as_function() {
        let _: fn(std::sync::Arc<std::sync::Mutex<crate::scene::WindowScene>>) =
            super::run_event_loop;
    }
}

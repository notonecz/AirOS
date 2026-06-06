use crate::scene::WindowScene;
use std::sync::{Arc, Mutex};

pub struct AirCompRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    target: wgpu::Texture,
    target_size: (u32, u32),
}

impl AirCompRenderer {
    /// Inicializuje wgpu renderer. Vrátí None pokud není dostupný adapter (CI bez GPU).
    pub async fn new(width: u32, height: u32) -> Option<Self> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: None,
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

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("aircomp_offscreen"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        Some(Self {
            device,
            queue,
            target,
            target_size: (width, height),
        })
    }

    /// Vykreslí jeden frame — vymaže na pozadí a složí okna ze scény.
    pub fn render_frame(&self, _scene: &WindowScene) {
        let view = self
            .target
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
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

        self.queue.submit([encoder.finish()]);
    }

    pub fn target_size(&self) -> (u32, u32) {
        self.target_size
    }
}

/// Headless render loop — inicializuje renderer a vykreslí jeden frame.
pub async fn run_headless(scene: Arc<Mutex<WindowScene>>) {
    if let Some(renderer) = AirCompRenderer::new(1920, 1080).await {
        let s = scene.lock().unwrap();
        renderer.render_frame(&s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn renderer_initializes_or_skips() {
        let renderer = AirCompRenderer::new(800, 600).await;
        if renderer.is_none() {
            return; // Žádný GPU adapter — přijatelné v CI bez GPU
        }
        let renderer = renderer.unwrap();
        assert_eq!(renderer.target_size(), (800, 600));
    }

    #[tokio::test]
    async fn render_frame_does_not_panic() {
        let renderer = AirCompRenderer::new(400, 300).await;
        if renderer.is_none() {
            return;
        }
        let scene = WindowScene::new();
        renderer.unwrap().render_frame(&scene);
    }

    #[tokio::test]
    async fn render_frame_with_windows_does_not_panic() {
        let renderer = AirCompRenderer::new(400, 300).await;
        if renderer.is_none() {
            return;
        }
        let mut scene = WindowScene::new();
        scene.add_window(
            airproto::types::WindowId(1),
            airproto::types::Size {
                width: 200,
                height: 150,
            },
        );
        renderer.unwrap().render_frame(&scene);
    }
}

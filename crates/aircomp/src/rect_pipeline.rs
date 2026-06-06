// crates/aircomp/src/rect_pipeline.rs

/// Barevný obdélník v pixelových souřadnicích (origin = levý horní roh obrazovky).
#[derive(Debug, Clone, Copy)]
pub struct ColoredRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// RGBA barva v rozsahu 0.0..=1.0.
    pub color: [f32; 4],
}

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
                entry_point: "vs_main",
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
                entry_point: "fs_main",
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
    pub fn update_screen_size(&self, queue: &wgpu::Queue, w: f32, h: f32) {
        let data: [f32; 2] = [w, h];
        let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_ne_bytes()).collect();
        queue.write_buffer(&self.screen_buf, 0, &bytes);
    }

    /// Nahraje recty do GPU a vykreslí je do `view` (LoadOp::Load — additivní na clear pass).
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

        let mut data: Vec<f32> = Vec::with_capacity(rects.len() * 8);
        for r in rects {
            data.extend_from_slice(&[r.x, r.y, r.w, r.h]);
            data.extend_from_slice(&r.color);
        }
        let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_ne_bytes()).collect();
        queue.write_buffer(&self.vertex_buf, 0, &bytes);

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

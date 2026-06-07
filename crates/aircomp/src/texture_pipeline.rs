// crates/aircomp/src/texture_pipeline.rs

/// Pixel-space souřadnice a RGBA data jednoho app okna pro render.
#[derive(Debug)]
pub struct WindowDraw<'a> {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub pixels: &'a [u8], // RGBA, délka = w*h*4
}

const SHADER_SRC: &str = r#"
struct ScreenSize {
    width: f32,
    height: f32,
};

struct RectUniform {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
};

@group(0) @binding(0) var<uniform> screen: ScreenSize;
@group(0) @binding(1) var<uniform> rect: RectUniform;

@group(1) @binding(0) var t_window: texture_2d<f32>;
@group(1) @binding(1) var s_window: sampler;

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let c = corners[vi];
    let px = rect.x + c.x * rect.w;
    let py = rect.y + c.y * rect.h;
    let ndcx = px / screen.width * 2.0 - 1.0;
    let ndcy = 1.0 - py / screen.height * 2.0;
    var out: VertexOut;
    out.pos = vec4<f32>(ndcx, ndcy, 0.0, 1.0);
    out.uv = c;
    return out;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(t_window, s_window, uv);
}
"#;

pub struct TexturePipeline {
    pipeline: wgpu::RenderPipeline,
    screen_buf: wgpu::Buffer,
    rect_buf: wgpu::Buffer,
    globals_bg: wgpu::BindGroup,
    texture_bgl: wgpu::BindGroupLayout,
    texture_format: wgpu::TextureFormat,
}

impl TexturePipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("texture_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let screen_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tex_screen_size"),
            size: 8, // 2 × f32
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rect_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tex_rect"),
            size: 16, // 4 × f32
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tex_globals_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let globals_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tex_globals_bg"),
            layout: &globals_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rect_buf.as_entire_binding(),
                },
            ],
        });

        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tex_texture_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tex_layout"),
            bind_group_layouts: &[&globals_bgl, &texture_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
            screen_buf,
            rect_buf,
            globals_bg,
            texture_bgl,
            texture_format: format,
        }
    }

    /// Aktualizuje uniform buffer s rozlišením obrazovky.
    pub fn update_screen_size(&self, queue: &wgpu::Queue, w: f32, h: f32) {
        let data: [f32; 2] = [w, h];
        let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_ne_bytes()).collect();
        queue.write_buffer(&self.screen_buf, 0, &bytes);
    }

    /// Vykreslí jedno app okno jako texturu.
    /// pixels musí být RGBA data délky w*h*4.
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        win: &WindowDraw<'_>,
    ) {
        assert_eq!(
            win.pixels.len(),
            win.w as usize * win.h as usize * 4,
            "pixels length does not match w*h*4"
        );

        // Upload rect uniform
        let rect_data: [f32; 4] = [win.x, win.y, win.w, win.h];
        let rect_bytes: Vec<u8> = rect_data.iter().flat_map(|f| f.to_ne_bytes()).collect();
        queue.write_buffer(&self.rect_buf, 0, &rect_bytes);

        // Vytvořit texturu pro toto okno
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("window_texture"),
            size: wgpu::Extent3d {
                width: win.w as u32,
                height: win.h as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            win.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(win.w as u32 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: win.w as u32,
                height: win.h as u32,
                depth_or_array_layers: 1,
            },
        );

        let tex_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("window_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("window_bg"),
            layout: &self.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("texture_pass"),
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
        pass.set_bind_group(0, &self.globals_bg, &[]);
        pass.set_bind_group(1, &texture_bg, &[]);
        pass.draw(0..6, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        adapter
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
            .ok()
    }

    #[test]
    fn window_draw_struct_exists() {
        let pixels = vec![255u8; 4];
        let _draw = WindowDraw {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            pixels: &pixels,
        };
    }

    #[tokio::test]
    async fn texture_pipeline_creates_without_panic() {
        let Some((device, _queue)) = make_device().await else {
            return; // Žádný GPU adapter — OK v CI
        };
        let _pipeline = TexturePipeline::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    }
}

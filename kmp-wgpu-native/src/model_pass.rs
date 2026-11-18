//! ModelPass — 独立网格 Present（非 FullscreenPass）。Android 先通。

use crate::error_bridge::{install_uncaptured_handler, ErrorSink, CODE_SURFACE};
use crate::fullscreen_pass::backend_label;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelFrameUniforms {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub light_dir: [f32; 4], // xyz + lit in w
}

unsafe impl bytemuck::Pod for ModelFrameUniforms {}
unsafe impl bytemuck::Zeroable for ModelFrameUniforms {}

const MODEL_WGSL: &str = r#"
struct Uniforms {
  mvp: mat4x4<f32>,
  color: vec4<f32>,
  light_dir: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsIn {
  @location(0) pos: vec3<f32>,
  @location(1) nrm: vec3<f32>,
};
struct VsOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) nrm: vec3<f32>,
};

@vertex
fn vs_main(v: VsIn) -> VsOut {
  var o: VsOut;
  o.clip = u.mvp * vec4<f32>(v.pos, 1.0);
  o.nrm = v.nrm;
  return o;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
  if (u.light_dir.w < 0.5) {
    return u.color;
  }
  let n = normalize(i.nrm);
  let l = normalize(u.light_dir.xyz);
  let ndl = max(dot(n, l), 0.15);
  return vec4<f32>(u.color.rgb * ndl, u.color.a);
}
"#;

pub struct ModelSessionCore {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub depth_tex: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buf: Option<wgpu::Buffer>,
    pub index_buf: Option<wgpu::Buffer>,
    pub index_count: u32,
    pub frame: ModelFrameUniforms,
    pub playing: bool,
    pub backend_id: &'static str,
    pub error_sink: ErrorSink,
}

impl ModelSessionCore {
    pub async fn create_with_surface(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
        downlevel_limits: bool,
    ) -> Result<Self, String> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("no wgpu adapter: {e}"))?;

        let backend_id = backend_label(adapter.get_info().backend);
        let adapter_limits = adapter.limits();
        let required_limits = if downlevel_limits {
            wgpu::Limits::downlevel_webgl2_defaults()
                .using_resolution(adapter_limits.clone())
                .or_worse_values_from(&adapter_limits)
        } else {
            wgpu::Limits::default()
                .using_resolution(adapter_limits.clone())
                .or_worse_values_from(&adapter_limits)
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("kmp-wgpu-model-session"),
                required_features: wgpu::Features::empty(),
                required_limits,
                memory_hints: Default::default(),
                trace: Default::default(),
                experimental_features: Default::default(),
            })
            .await
            .map_err(|e| format!("request_device: {e}"))?;

        let error_sink = ErrorSink::new();
        install_uncaptured_handler(&device, error_sink.clone());

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.iter().copied().find(|f| f.is_srgb()))
            .or_else(|| caps.formats.first().copied())
            .ok_or_else(|| "no surface formats".to_string())?;

        let present_mode = if caps
            .present_modes
            .iter()
            .any(|m| *m == wgpu::PresentMode::AutoNoVsync)
        {
            wgpu::PresentMode::AutoNoVsync
        } else {
            caps.present_modes
                .iter()
                .copied()
                .find(|m| {
                    matches!(
                        *m,
                        wgpu::PresentMode::Immediate
                            | wgpu::PresentMode::Mailbox
                            | wgpu::PresentMode::FifoRelaxed
                    )
                })
                .or_else(|| caps.present_modes.first().copied())
                .unwrap_or(wgpu::PresentMode::Fifo)
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        let (depth_tex, depth_view) = Self::make_depth(&device, config.width, config.height);

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kmp-model-uniforms"),
            size: std::mem::size_of::<ModelFrameUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("kmp-model-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kmp-model-bg"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });
        let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("kmp-model-pipe-layout"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("kmp-model-shader"),
            source: wgpu::ShaderSource::Wgsl(MODEL_WGSL.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("kmp-model-pipeline"),
            layout: Some(&pipe_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 24,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let mut identity = [[0.0f32; 4]; 4];
        identity[0][0] = 1.0;
        identity[1][1] = 1.0;
        identity[2][2] = 1.0;
        identity[3][3] = 1.0;

        Ok(Self {
            device,
            queue,
            surface,
            config,
            depth_tex,
            depth_view,
            uniform_buf,
            bind_group,
            pipeline,
            vertex_buf: None,
            index_buf: None,
            index_count: 0,
            frame: ModelFrameUniforms {
                mvp: identity,
                color: [0.85, 0.55, 0.25, 1.0],
                light_dir: [0.3, 0.8, 0.4, 0.0],
            },
            playing: true,
            backend_id,
            error_sink,
        })
    }

    fn make_depth(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("kmp-model-depth"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        if self.config.width == w && self.config.height == h {
            return;
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        let (tex, view) = Self::make_depth(&self.device, w, h);
        self.depth_tex = tex;
        self.depth_view = view;
    }

    pub fn set_meshes(&mut self, verts: &[f32], indices: &[u32]) -> Result<(), String> {
        if verts.is_empty() || indices.is_empty() {
            self.vertex_buf = None;
            self.index_buf = None;
            self.index_count = 0;
            return Ok(());
        }
        if verts.len() % 6 != 0 {
            return Err("verts must be interleaved pos3+nrm3".into());
        }
        let vbuf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kmp-model-vbo"),
            size: (verts.len() * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&vbuf, 0, bytemuck::cast_slice(verts));
        let ibuf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kmp-model-ibo"),
            size: (indices.len() * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&ibuf, 0, bytemuck::cast_slice(indices));
        self.vertex_buf = Some(vbuf);
        self.index_buf = Some(ibuf);
        self.index_count = indices.len() as u32;
        Ok(())
    }

    pub fn set_frame(
        &mut self,
        mvp: &[f32; 16],
        color: &[f32; 4],
        lit: bool,
        light_dir: &[f32; 3],
    ) {
        // column-major mat4
        for c in 0..4 {
            for r in 0..4 {
                self.frame.mvp[c][r] = mvp[c * 4 + r];
            }
        }
        self.frame.color = *color;
        self.frame.light_dir = [
            light_dir[0],
            light_dir[1],
            light_dir[2],
            if lit { 1.0 } else { 0.0 },
        ];
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn last_error(&self) -> (String, String) {
        self.error_sink.last()
    }

    pub fn drain_errors(&self) -> Vec<(String, String)> {
        self.error_sink.drain()
    }

    pub fn tick(&mut self, _dt_secs: f32) -> Result<(), String> {
        if !self.playing {
            return Ok(());
        }
        self.queue
            .write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&self.frame));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                t
            }
            other => {
                let msg = format!("get_current_texture: {other:?}");
                self.error_sink.emit(CODE_SURFACE, &msg);
                return Err(msg);
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("kmp-model-tick"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("kmp-model-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.08,
                            b: 0.18,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let (Some(vb), Some(ib)) = (self.vertex_buf.as_ref(), self.index_buf.as_ref()) {
                if self.index_count > 0 {
                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, Some(&self.bind_group), &[]);
                    pass.set_vertex_buffer(0, vb.slice(..));
                    pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..self.index_count, 0, 0..1);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        Ok(())
    }
}

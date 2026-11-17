//! FullscreenPass（ADR 0015）— 通用全屏片元 Present。
//!
//! 持有 wgpu Device/Queue/Surface；`set_fragment_shader` / playing / tick。
//! 无片元时 tick 只清屏（不画 ClearTri）。liquid / scene / ClearTri 已废。

use crate::error_bridge::{
    install_uncaptured_handler, validate_wgsl_module, ErrorSink, CODE_SHADER_CUSTOM, CODE_SURFACE,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::error_bridge::{classify_wgpu_error, CODE_WGPU_INTERNAL};
use crate::shaders;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FrameUniforms {
    pub time: f32,
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub resolution: [f32; 2],
    pub _pad3: [f32; 2],
}

unsafe impl bytemuck::Pod for FrameUniforms {}
unsafe impl bytemuck::Zeroable for FrameUniforms {}

pub fn backend_label(backend: wgpu::Backend) -> &'static str {
    match backend {
        wgpu::Backend::BrowserWebGpu => "wgpu-m0",
        wgpu::Backend::Gl => "wgpu-m0-gl",
        wgpu::Backend::Vulkan => "wgpu-m0",
        _ => "wgpu-m0",
    }
}

pub struct FullscreenSessionCore {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub pipe_layout: wgpu::PipelineLayout,
    pub pipe_custom: Option<wgpu::RenderPipeline>,
    pub playing: bool,
    pub time: f32,
    pub backend_id: &'static str,
    pub error_sink: ErrorSink,
}

impl FullscreenSessionCore {
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
                label: Some("kmp-wgpu-native-session"),
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

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kmp-frame-uniforms"),
            size: std::mem::size_of::<FrameUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("kmp-frame-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kmp-frame-bg"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });
        let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("kmp-pipe-layout"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });

        Ok(Self {
            device,
            queue,
            surface,
            config,
            uniform_buf,
            bind_group,
            pipe_layout,
            pipe_custom: None,
            playing: true,
            time: 0.0,
            backend_id,
            error_sink,
        })
    }

    pub fn build_pipe(
        device: &wgpu::Device,
        pipe_layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        label: &str,
        fs: &str,
    ) -> Result<wgpu::RenderPipeline, String> {
        let src = shaders::module_source(fs);
        validate_wgsl_module(&src)?;

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });
        let pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(pipe_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        #[cfg(not(target_arch = "wasm32"))]
        {
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| format!("{CODE_WGPU_INTERNAL}: device poll: {e}"))?;
            if let Some(err) = pollster::block_on(error_scope.pop()) {
                let (code, msg) = classify_wgpu_error(&err);
                return Err(format!("{code}: {msg}"));
            }
        }

        Ok(pipe)
    }

    pub fn set_fragment_shader(&mut self, fs: &str) -> Result<(), String> {
        let trimmed = fs.trim();
        if trimmed.is_empty() {
            self.pipe_custom = None;
            return Ok(());
        }
        match Self::build_pipe(
            &self.device,
            &self.pipe_layout,
            self.config.format,
            "kmp-fragment",
            trimmed,
        ) {
            Ok(pipe) => {
                self.pipe_custom = Some(pipe);
                Ok(())
            }
            Err(e) => {
                self.pipe_custom = None;
                self.error_sink.emit(CODE_SHADER_CUSTOM, &e);
                let _ = self.present_clear_only();
                self.playing = false;
                Err(e)
            }
        }
    }

    fn present_clear_only(&mut self) -> Result<(), String> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                t
            }
            other => {
                return Err(format!("get_current_texture: {other:?}"));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("kmp-clear-only"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("kmp-clear-only-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.06,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        Ok(())
    }

    pub fn last_error(&self) -> (String, String) {
        self.error_sink.last()
    }

    pub fn drain_errors(&self) -> Vec<(String, String)> {
        self.error_sink.drain()
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
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
    }

    pub fn tick(&mut self, dt_secs: f32) -> Result<(), String> {
        if !self.playing {
            return Ok(());
        }
        let Some(pipeline) = self.pipe_custom.as_ref() else {
            return self.present_clear_only();
        };
        self.time += dt_secs.max(0.0);
        let uniforms = FrameUniforms {
            time: self.time,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            resolution: [self.config.width as f32, self.config.height as f32],
            _pad3: [0.0, 0.0],
        };
        self.queue
            .write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));

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
                label: Some("kmp-tick"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("kmp-tick-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.06,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, Some(&self.bind_group), &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        Ok(())
    }
}

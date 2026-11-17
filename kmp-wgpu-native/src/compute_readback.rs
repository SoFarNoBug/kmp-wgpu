//! Headless Compute + 回读（ADR 0013 · v1.1 · 方案 B 粗块）。
//!
//! 无 Surface / 无 Holm 全屏语义；一次 `run_f32` = pipeline + dispatch + map 回读。

use std::borrow::Cow;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;

use crate::fullscreen_pass::backend_label;

/// 默认示例：`out[i] = in[i] * 2`（`@workgroup_size(64)`）。
/// 与 Kotlin `COMPUTE_SAMPLE_DOUBLE_WGSL`（`kmp-wgpu/.../ComputeReadback.kt`）**双份**；改一处须同步另一处。
pub const SAMPLE_DOUBLE_WGSL: &str = r#"
@group(0) @binding(0) var<storage, read> input_buf: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_buf: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&input_buf)) {
        return;
    }
    output_buf[i] = input_buf[i] * 2.0;
}
"#;

pub struct HeadlessCompute {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub backend_id: &'static str,
}

impl HeadlessCompute {
    pub async fn create(downlevel_limits: bool) -> Result<Self, String> {
        #[cfg(target_os = "android")]
        let instance = {
            // Android headless：无 Surface 时仍可走 Vulkan|GL
            let mut desc =
                wgpu::InstanceDescriptor::new_with_display_handle(Box::new(AndroidDisplayOwner));
            desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
            wgpu::Instance::new(desc)
        };
        #[cfg(target_env = "ohos")]
        let instance = {
            // Ohos headless：与 Present 同需 DisplayHandle 标记（target_os=linux + env=ohos）
            let mut desc =
                wgpu::InstanceDescriptor::new_with_display_handle(Box::new(OhosDisplayOwner));
            desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
            wgpu::Instance::new(desc)
        };
        #[cfg(target_os = "ios")]
        let instance = {
            // iOS headless：无 CAMetalLayer；仅 Metal
            let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
            desc.backends = wgpu::Backends::METAL;
            wgpu::Instance::new(desc)
        };
        #[cfg(target_arch = "wasm32")]
        let instance = {
            let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
            desc.backends = wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL;
            wgpu::Instance::new(desc)
        };
        #[cfg(all(
            not(target_os = "android"),
            not(target_env = "ohos"),
            not(target_os = "ios"),
            not(target_arch = "wasm32")
        ))]
        let instance = {
            let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
            desc.backends = wgpu::Backends::all();
            wgpu::Instance::new(desc)
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("no wgpu adapter (headless): {e}"))?;

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
                label: Some("kmp-wgpu-headless"),
                required_features: wgpu::Features::empty(),
                required_limits,
                memory_hints: Default::default(),
                trace: Default::default(),
                experimental_features: Default::default(),
            })
            .await
            .map_err(|e| format!("request_device (headless): {e}"))?;

        Ok(Self {
            device,
            queue,
            backend_id,
        })
    }

    /// 粗块：STORAGE in/out + compute dispatch + 同步回读。
    pub async fn run_f32(
        &self,
        wgsl: &str,
        input: &[f32],
        workgroup_count_x: u32,
    ) -> Result<Vec<f32>, String> {
        if input.is_empty() {
            return Err("compute_readback: empty input".into());
        }
        let wg_x = workgroup_count_x.max(1);
        let byte_len = (input.len() * std::mem::size_of::<f32>()) as u64;

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("kmp-compute-cs"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(wgsl)),
            });

        let bgl = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("kmp-compute-bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("kmp-compute-pl"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("kmp-compute-pipe"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        let in_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kmp-compute-in"),
            size: byte_len,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&in_buf, 0, bytemuck::cast_slice(input));

        let out_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kmp-compute-out"),
            size: byte_len,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kmp-compute-staging"),
            size: byte_len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kmp-compute-bg"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: in_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: out_buf.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("kmp-compute-enc"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("kmp-compute-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(wg_x, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, byte_len);
        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        map_buffer_sync(&self.device, slice).await?;
        let data = slice
            .get_mapped_range()
            .map_err(|e| format!("compute_readback: get_mapped_range: {e}"))?;
        let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging.unmap();
        if out.len() != input.len() {
            return Err(format!(
                "compute_readback: length mismatch out={} in={}",
                out.len(),
                input.len()
            ));
        }
        Ok(out)
    }
}

async fn map_buffer_sync(
    device: &wgpu::Device,
    slice: wgpu::BufferSlice<'_>,
) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()
            .map_err(|_| "compute_readback: map channel closed".to_string())?
            .map_err(|e| format!("compute_readback: map_async: {e}"))
    }
    #[cfg(target_arch = "wasm32")]
    {
        use std::cell::RefCell;
        use std::rc::Rc;
        let done: Rc<RefCell<Option<Result<(), wgpu::BufferAsyncError>>>> =
            Rc::new(RefCell::new(None));
        let done2 = done.clone();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            *done2.borrow_mut() = Some(r);
        });
        let mut spins = 0u32;
        loop {
            let _ = device.poll(wgpu::PollType::Poll);
            if let Some(r) = done.borrow_mut().take() {
                return r.map_err(|e| format!("compute_readback: map_async: {e}"));
            }
            spins += 1;
            if spins > 10_000 {
                return Err("compute_readback: map_async timeout".into());
            }
            // 让出事件循环，避免卡死浏览器
            let p = js_sys::Promise::new(&mut |resolve, _reject| {
                let _ = web_sys::window().map(|w| {
                    let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0);
                });
            });
            let _ = wasm_bindgen_futures::JsFuture::from(p).await;
        }
    }
}

#[cfg(target_os = "android")]
#[derive(Debug)]
struct AndroidDisplayOwner;

#[cfg(target_os = "android")]
impl raw_window_handle::HasDisplayHandle for AndroidDisplayOwner {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(unsafe {
            raw_window_handle::DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Android(
                raw_window_handle::AndroidDisplayHandle::new(),
            ))
        })
    }
}

#[cfg(target_env = "ohos")]
#[derive(Debug)]
struct OhosDisplayOwner;

#[cfg(target_env = "ohos")]
impl raw_window_handle::HasDisplayHandle for OhosDisplayOwner {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(unsafe {
            raw_window_handle::DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Ohos(
                raw_window_handle::OhosDisplayHandle::new(),
            ))
        })
    }
}

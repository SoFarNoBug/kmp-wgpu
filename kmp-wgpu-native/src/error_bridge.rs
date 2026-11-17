//! Runtime 错误桥：WGSL 同步校验 + 错误码分类 + pending 队列（供宿主 drain → onError）。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 业务侧 `ErrorParams.code` 约定（与 Plan / Demo 一致）。
pub const CODE_SHADER_CUSTOM: &str = "SHADER_CUSTOM";
pub const CODE_WGPU_VALIDATION: &str = "WGPU_VALIDATION";
pub const CODE_WGPU_OOM: &str = "WGPU_OOM";
pub const CODE_WGPU_INTERNAL: &str = "WGPU_INTERNAL";
pub const CODE_WGPU_DEVICE_LOST: &str = "WGPU_DEVICE_LOST";
pub const CODE_TICK: &str = "TICK";
pub const CODE_SURFACE: &str = "SURFACE";

#[derive(Default)]
struct ErrorState {
    last_code: String,
    last_message: String,
    pending: VecDeque<(String, String)>,
}

/// 线程安全错误槽：uncaptured / set_fragment_shader / tick 共用。
#[derive(Clone, Default)]
pub struct ErrorSink {
    inner: Arc<Mutex<ErrorState>>,
}

impl ErrorSink {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ErrorState::default())),
        }
    }

    pub fn emit(&self, code: &str, message: &str) {
        if let Ok(mut g) = self.inner.lock() {
            g.last_code = code.to_string();
            g.last_message = message.to_string();
            g.pending.push_back((code.to_string(), message.to_string()));
            // 防止无限堆积
            while g.pending.len() > 32 {
                g.pending.pop_front();
            }
        }
    }

    pub fn last(&self) -> (String, String) {
        self.inner
            .lock()
            .map(|g| (g.last_code.clone(), g.last_message.clone()))
            .unwrap_or_default()
    }

    pub fn drain(&self) -> Vec<(String, String)> {
        self.inner
            .lock()
            .map(|mut g| g.pending.drain(..).collect())
            .unwrap_or_default()
    }
}

/// 同步解析完整 WGSL 模块源（含 VS + uniforms + 业务 FS）。失败文案给 SHADER_CUSTOM。
pub fn validate_wgsl_module(src: &str) -> Result<(), String> {
    naga::front::wgsl::parse_str(src).map_err(|e| format!("WGSL: {e}"))?;
    Ok(())
}

pub fn classify_wgpu_error(err: &wgpu::Error) -> (&'static str, String) {
    match err {
        wgpu::Error::Validation { description, .. } => {
            let desc = description.clone();
            let lower = desc.to_ascii_lowercase();
            if lower.contains("device") && lower.contains("lost") {
                (CODE_WGPU_DEVICE_LOST, desc)
            } else {
                (CODE_WGPU_VALIDATION, desc)
            }
        }
        wgpu::Error::OutOfMemory { .. } => (CODE_WGPU_OOM, "wgpu out of memory".to_string()),
        wgpu::Error::Internal { description, .. } => (CODE_WGPU_INTERNAL, description.clone()),
        #[allow(unreachable_patterns)]
        other => (CODE_WGPU_INTERNAL, format!("{other:?}")),
    }
}

pub fn install_uncaptured_handler(device: &wgpu::Device, sink: ErrorSink) {
    device.on_uncaptured_error(Arc::new(move |err| {
        let (code, msg) = classify_wgpu_error(&err);
        sink.emit(code, &msg);
    }));
}

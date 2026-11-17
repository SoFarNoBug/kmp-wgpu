//! H5 wasm 泵帧 Session：包装 [`crate::fullscreen_pass`]；HtmlCanvas / OffscreenCanvas。
//! 另：headless Compute（v1.1）[`HeadlessComputeSession`]，无 canvas。

use crate::compute_readback::{HeadlessCompute, SAMPLE_DOUBLE_WGSL};
use crate::fullscreen_pass::FullscreenSessionCore;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, OffscreenCanvas};

#[wasm_bindgen]
pub struct KmpWgpuSession {
    inner: FullscreenSessionCore,
}

#[wasm_bindgen]
impl KmpWgpuSession {
    /// 异步创建 Session；`target` 为 `HtmlCanvasElement` 或 `OffscreenCanvas`。
    pub async fn create(target: JsValue) -> Result<KmpWgpuSession, JsValue> {
        console_error_panic_hook::set_once();
        if let Ok(canvas) = target.clone().dyn_into::<HtmlCanvasElement>() {
            let width = canvas.width().max(1);
            let height = canvas.height().max(1);
            return Self::create_with_surface(
                wgpu::SurfaceTarget::Canvas(canvas),
                width,
                height,
            )
            .await;
        }
        if let Ok(canvas) = target.dyn_into::<OffscreenCanvas>() {
            let width = canvas.width().max(1);
            let height = canvas.height().max(1);
            return Self::create_with_surface(
                wgpu::SurfaceTarget::OffscreenCanvas(canvas),
                width,
                height,
            )
            .await;
        }
        Err(JsValue::from_str(
            "create: target must be HtmlCanvasElement or OffscreenCanvas",
        ))
    }

    async fn create_with_surface(
        target: wgpu::SurfaceTarget<'static>,
        width: u32,
        height: u32,
    ) -> Result<KmpWgpuSession, JsValue> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL;
        let instance = wgpu::Instance::new(desc);
        let surface = instance
            .create_surface(target)
            .map_err(|e| JsValue::from_str(&format!("create_surface: {e}")))?;
        let inner = FullscreenSessionCore::create_with_surface(instance, surface, width, height, true)
            .await
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { inner })
    }

    #[wasm_bindgen(js_name = backendId)]
    pub fn backend_id(&self) -> String {
        self.inner.backend_id.to_string()
    }

    /// 业务 WGSL 片元正文（须含 `@fragment fn fs_main`）；空串清除后 tick 只清屏。
    #[wasm_bindgen(js_name = setFragmentShader)]
    pub fn set_fragment_shader(&mut self, fs: &str) -> Result<(), JsValue> {
        self.inner
            .set_fragment_shader(fs)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = setPlaying)]
    pub fn set_playing(&mut self, playing: bool) {
        self.inner.set_playing(playing);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.inner.resize(width, height);
    }

    /// 推进一帧；`dt_secs` 为秒。未 playing 时跳过 GPU（保留上一帧）。
    pub fn tick(&mut self, dt_secs: f32) -> Result<(), JsValue> {
        self.inner
            .tick(dt_secs)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// 最近一次错误码（setFragmentShader / tick / uncaptured）。
    #[wasm_bindgen(js_name = lastErrorCode)]
    pub fn last_error_code(&self) -> String {
        self.inner.last_error().0
    }

    /// 最近一次错误文案。
    #[wasm_bindgen(js_name = lastErrorMessage)]
    pub fn last_error_message(&self) -> String {
        self.inner.last_error().1
    }

    /// 取出 pending 错误队列（含 uncaptured）；元素为 `{code, message}`。
    #[wasm_bindgen(js_name = drainErrors)]
    pub fn drain_errors(&self) -> js_sys::Array {
        let out = js_sys::Array::new();
        for (code, message) in self.inner.drain_errors() {
            let obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("code"), &JsValue::from_str(&code));
            let _ = js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("message"),
                &JsValue::from_str(&message),
            );
            out.push(&obj);
        }
        out
    }

    /// 释放 Session（宿主卸岛时调用）。
    pub fn destroy(self) {
        // drop inner
    }
}

/// Headless Compute + 回读（ADR 0013 v1.1）；与 [`KmpWgpuSession`] 分型，勿混用。
#[wasm_bindgen]
pub struct HeadlessComputeSession {
    inner: HeadlessCompute,
}

#[wasm_bindgen]
impl HeadlessComputeSession {
    /// 无 canvas 建 Device/Queue（WebGPU / WebGL 回退由实例 backends 决定）。
    pub async fn create() -> Result<HeadlessComputeSession, JsValue> {
        console_error_panic_hook::set_once();
        let inner = HeadlessCompute::create(true)
            .await
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Self { inner })
    }

    #[wasm_bindgen(js_name = backendId)]
    pub fn backend_id(&self) -> String {
        self.inner.backend_id.to_string()
    }

    /// 默认示例 WGSL：`out[i] = in[i] * 2`。
    #[wasm_bindgen(js_name = sampleDoubleWgsl)]
    pub fn sample_double_wgsl() -> String {
        SAMPLE_DOUBLE_WGSL.to_string()
    }

    /// 粗块：WGSL compute + Float32Array 回读。
    #[wasm_bindgen(js_name = runF32)]
    pub async fn run_f32(
        &self,
        wgsl: &str,
        input: js_sys::Float32Array,
        workgroup_count_x: u32,
    ) -> Result<js_sys::Float32Array, JsValue> {
        let mut buf = vec![0f32; input.length() as usize];
        input.copy_to(&mut buf);
        let out = self
            .inner
            .run_f32(wgsl, &buf, workgroup_count_x.max(1))
            .await
            .map_err(|e| JsValue::from_str(&e))?;
        let arr = js_sys::Float32Array::new_with_length(out.len() as u32);
        arr.copy_from(&out);
        Ok(arr)
    }

    pub fn destroy(self) {
        // drop
    }
}

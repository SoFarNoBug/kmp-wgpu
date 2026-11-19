//! iOS C Session ABI（ADR 0009）：CAMetalLayer → wgpu Metal Surface；宿主外泵 tick。
//! 另：headless Compute（ADR 0013 v1.1）与 ModelPass 与 Present Session 分表。
//! 所有对外 C 入口 `catch_unwind`：非法 WGSL 等 panic 不得穿 FFI 把进程 abort。

use crate::compute_readback::HeadlessCompute;
use crate::fullscreen_pass::FullscreenSessionCore;
use crate::model_pass::ModelSessionCore;
use std::ffi::{c_char, CStr};
use std::os::raw::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::Mutex;

static LAST_CREATE_ERROR: Mutex<String> = Mutex::new(String::new());

fn set_last_create_error(msg: impl Into<String>) {
    if let Ok(mut g) = LAST_CREATE_ERROR.lock() {
        *g = msg.into();
    }
}

fn clear_last_create_error() {
    if let Ok(mut g) = LAST_CREATE_ERROR.lock() {
        g.clear();
    }
}

struct NativeSession {
    core: FullscreenSessionCore,
}

fn box_session(session: NativeSession) -> u64 {
    Box::into_raw(Box::new(session)) as u64
}

unsafe fn from_handle<'a>(handle: u64) -> Option<&'a mut NativeSession> {
    if handle == 0 {
        return None;
    }
    Some(&mut *(handle as *mut NativeSession))
}

/// Create Session bound to a CAMetalLayer. Layer must outlive the session; Rust does not retain it.
#[no_mangle]
pub extern "C" fn kmp_wgpu_session_create_metal(
    metal_layer: *mut c_void,
    width: u32,
    height: u32,
) -> u64 {
    if metal_layer.is_null() {
        set_last_create_error("metal_layer is null");
        return 0;
    }
    let w = width.max(1);
    let h = height.max(1);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::METAL;
        let instance = wgpu::Instance::new(desc);
        // SAFETY: caller guarantees a live CAMetalLayer* for the session lifetime.
        let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(metal_layer);
        let surface = unsafe { instance.create_surface_unsafe(target) }
            .map_err(|e| format!("create_surface: {e}"))?;
        let core = pollster::block_on(FullscreenSessionCore::create_with_surface(
            instance, surface, w, h, false,
        ))?;
        Ok::<FullscreenSessionCore, String>(core)
    }));

    match result {
        Ok(Ok(core)) => {
            clear_last_create_error();
            box_session(NativeSession { core })
        }
        Ok(Err(e)) => {
            set_last_create_error(e);
            0
        }
        Err(panic) => {
            set_last_create_error(format!("create_metal panic: {panic:?}"));
            0
        }
    }
}

/// Last create_metal failure reason (NUL-terminated UTF-8). Empty if last create succeeded.
#[no_mangle]
pub extern "C" fn kmp_wgpu_last_create_error(out_buf: *mut c_char, out_len: u32) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let msg = LAST_CREATE_ERROR
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        copy_utf8(&msg, out_buf, out_len)
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_destroy(handle: u64) {
    if handle == 0 {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(handle as *mut NativeSession));
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_resize(handle: u64, width: u32, height: u32) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(s) = unsafe { from_handle(handle) } {
            s.core.resize(width.max(1), height.max(1));
        }
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_set_fragment_shader(handle: u64, fs: *const c_char) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(s) = (unsafe { from_handle(handle) }) else {
            return 0u32;
        };
        let src = if fs.is_null() {
            ""
        } else {
            match unsafe { CStr::from_ptr(fs) }.to_str() {
                Ok(v) => v,
                Err(_) => return 0u32,
            }
        };
        match s.core.set_fragment_shader(src) {
            Ok(()) => 1u32,
            Err(_) => 0u32,
        }
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_set_playing(handle: u64, playing: u32) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(s) = unsafe { from_handle(handle) } {
            s.core.set_playing(playing != 0);
        }
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_tick(handle: u64, dt_secs: f32) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(s) = (unsafe { from_handle(handle) }) else {
            return 0u32;
        };
        match s.core.tick(dt_secs) {
            Ok(()) => 1u32,
            Err(_) => 0u32,
        }
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_backend_id(
    handle: u64,
    out_buf: *mut c_char,
    out_len: u32,
) -> u32 {
    if out_buf.is_null() || out_len == 0 {
        return 0;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let id = unsafe { from_handle(handle) }
            .map(|s| s.core.backend_id)
            .unwrap_or("wgpu-blocked");
        let bytes = id.as_bytes();
        let max = (out_len as usize).saturating_sub(1);
        let n = bytes.len().min(max);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, n);
            *out_buf.add(n) = 0;
        }
        n as u32
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

fn copy_utf8(s: &str, out_buf: *mut c_char, out_len: u32) -> u32 {
    if out_buf.is_null() || out_len == 0 {
        return 0;
    }
    let bytes = s.as_bytes();
    let max = (out_len as usize).saturating_sub(1);
    let n = bytes.len().min(max);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, n);
        *out_buf.add(n) = 0;
    }
    n as u32
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_last_error_code(
    handle: u64,
    out_buf: *mut c_char,
    out_len: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let (code, _) = unsafe { from_handle(handle) }
            .map(|s| s.core.last_error())
            .unwrap_or_default();
        copy_utf8(&code, out_buf, out_len)
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_session_last_error_message(
    handle: u64,
    out_buf: *mut c_char,
    out_len: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let (_, msg) = unsafe { from_handle(handle) }
            .map(|s| s.core.last_error())
            .unwrap_or_default();
        copy_utf8(&msg, out_buf, out_len)
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

// --- Headless Compute（v1.1）· 与 NativeSession 分表 ---

fn box_headless(session: HeadlessCompute) -> u64 {
    Box::into_raw(Box::new(session)) as u64
}

unsafe fn from_headless<'a>(handle: u64) -> Option<&'a mut HeadlessCompute> {
    if handle == 0 {
        return None;
    }
    Some(&mut *(handle as *mut HeadlessCompute))
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_headless_create() -> u64 {
    let created = catch_unwind(AssertUnwindSafe(|| {
        pollster::block_on(HeadlessCompute::create(false))
    }));
    match created {
        Ok(Ok(session)) => {
            clear_last_create_error();
            box_headless(session)
        }
        Ok(Err(e)) => {
            set_last_create_error(e);
            0
        }
        Err(panic) => {
            set_last_create_error(format!("headless_create panic: {panic:?}"));
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_headless_destroy(handle: u64) {
    if handle == 0 {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(handle as *mut HeadlessCompute));
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_headless_backend_id(
    handle: u64,
    out_buf: *mut c_char,
    out_len: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let id = unsafe { from_headless(handle) }
            .map(|s| s.backend_id)
            .unwrap_or("wgpu-blocked");
        copy_utf8(id, out_buf, out_len)
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_headless_compute_readback(
    handle: u64,
    wgsl: *const c_char,
    input: *const f32,
    input_len: u32,
    workgroup_count_x: u32,
    out_floats: *mut f32,
    out_cap: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(session) = (unsafe { from_headless(handle) }) else {
            set_last_create_error("headless_compute_readback: bad handle");
            return 0u32;
        };
        if wgsl.is_null() || input.is_null() || out_floats.is_null() || input_len == 0 || out_cap == 0
        {
            set_last_create_error("headless_compute_readback: null/empty args");
            return 0u32;
        }
        if out_cap < input_len {
            set_last_create_error("headless_compute_readback: out_cap < input_len");
            return 0u32;
        }
        let Ok(src) = unsafe { CStr::from_ptr(wgsl) }.to_str() else {
            set_last_create_error("headless_compute_readback: bad wgsl utf8");
            return 0u32;
        };
        let input_slice = unsafe { std::slice::from_raw_parts(input, input_len as usize) };
        let wg = workgroup_count_x.max(1);
        match pollster::block_on(session.run_f32(src, input_slice, wg)) {
            Ok(out) => {
                let n = out.len().min(out_cap as usize);
                unsafe {
                    ptr::copy_nonoverlapping(out.as_ptr(), out_floats, n);
                }
                clear_last_create_error();
                n as u32
            }
            Err(e) => {
                set_last_create_error(e);
                0u32
            }
        }
    }));
    match result {
        Ok(v) => v,
        Err(panic) => {
            set_last_create_error(format!("headless_compute_readback panic: {panic:?}"));
            0
        }
    }
}

// --- ModelPass · 与 Fullscreen NativeSession / Headless 分表 ---

struct ModelNativeSession {
    core: ModelSessionCore,
}

fn box_model(session: ModelNativeSession) -> u64 {
    Box::into_raw(Box::new(session)) as u64
}

unsafe fn from_model<'a>(handle: u64) -> Option<&'a mut ModelNativeSession> {
    if handle == 0 {
        return None;
    }
    Some(&mut *(handle as *mut ModelNativeSession))
}

/// Create ModelPass bound to a CAMetalLayer. Layer must outlive the session; Rust does not retain it.
#[no_mangle]
pub extern "C" fn kmp_wgpu_model_create_metal(
    metal_layer: *mut c_void,
    width: u32,
    height: u32,
) -> u64 {
    if metal_layer.is_null() {
        set_last_create_error("metal_layer is null");
        return 0;
    }
    let w = width.max(1);
    let h = height.max(1);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::METAL;
        let instance = wgpu::Instance::new(desc);
        // SAFETY: caller guarantees a live CAMetalLayer* for the session lifetime.
        let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(metal_layer);
        let surface = unsafe { instance.create_surface_unsafe(target) }
            .map_err(|e| format!("create_surface: {e}"))?;
        let core = pollster::block_on(ModelSessionCore::create_with_surface(
            instance, surface, w, h, false,
        ))?;
        Ok::<ModelSessionCore, String>(core)
    }));

    match result {
        Ok(Ok(core)) => {
            clear_last_create_error();
            box_model(ModelNativeSession { core })
        }
        Ok(Err(e)) => {
            set_last_create_error(e);
            0
        }
        Err(panic) => {
            set_last_create_error(format!("model_create_metal panic: {panic:?}"));
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_destroy(handle: u64) {
    if handle == 0 {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(handle as *mut ModelNativeSession));
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_resize(handle: u64, width: u32, height: u32) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(s) = unsafe { from_model(handle) } {
            s.core.resize(width.max(1), height.max(1));
        }
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_set_meshes(
    handle: u64,
    verts: *const f32,
    vert_floats: u32,
    indices: *const i32,
    index_count: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(s) = (unsafe { from_model(handle) }) else {
            return 0u32;
        };
        // Empty mesh clear (mirrors ModelSessionCore + Android JNI).
        if vert_floats == 0 || index_count == 0 {
            return match s.core.set_meshes(&[], &[]) {
                Ok(()) => 1u32,
                Err(_) => 0u32,
            };
        }
        if verts.is_null() || indices.is_null() {
            return 0u32;
        }
        let verts_slice = unsafe { std::slice::from_raw_parts(verts, vert_floats as usize) };
        let indices_i32 = unsafe { std::slice::from_raw_parts(indices, index_count as usize) };
        let indices_u32: Vec<u32> = indices_i32.iter().map(|x| *x as u32).collect();
        match s.core.set_meshes(verts_slice, &indices_u32) {
            Ok(()) => 1u32,
            Err(_) => 0u32,
        }
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_set_frame(
    handle: u64,
    mvp16: *const f32,
    color4: *const f32,
    lit: u32,
    light_dir3: *const f32,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let Some(s) = (unsafe { from_model(handle) }) else {
            return;
        };
        if mvp16.is_null() || color4.is_null() || light_dir3.is_null() {
            return;
        }
        let mvp = unsafe { std::slice::from_raw_parts(mvp16, 16) };
        let color = unsafe { std::slice::from_raw_parts(color4, 4) };
        let light = unsafe { std::slice::from_raw_parts(light_dir3, 3) };
        let mut mvp_arr = [0.0f32; 16];
        mvp_arr.copy_from_slice(mvp);
        let mut color_arr = [0.0f32; 4];
        color_arr.copy_from_slice(color);
        let mut light_arr = [0.0f32; 3];
        light_arr.copy_from_slice(light);
        s.core
            .set_frame(&mvp_arr, &color_arr, lit != 0, &light_arr);
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_set_playing(handle: u64, playing: u32) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(s) = unsafe { from_model(handle) } {
            s.core.set_playing(playing != 0);
        }
    }));
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_tick(handle: u64, dt_secs: f32) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(s) = (unsafe { from_model(handle) }) else {
            return 0u32;
        };
        match s.core.tick(dt_secs) {
            Ok(()) => 1u32,
            Err(_) => 0u32,
        }
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_backend_id(
    handle: u64,
    out_buf: *mut c_char,
    out_len: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let id = unsafe { from_model(handle) }
            .map(|s| s.core.backend_id)
            .unwrap_or("wgpu-blocked");
        copy_utf8(id, out_buf, out_len)
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_last_error_code(
    handle: u64,
    out_buf: *mut c_char,
    out_len: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let (code, _) = unsafe { from_model(handle) }
            .map(|s| s.core.last_error())
            .unwrap_or_default();
        copy_utf8(&code, out_buf, out_len)
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn kmp_wgpu_model_last_error_message(
    handle: u64,
    out_buf: *mut c_char,
    out_len: u32,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let (_, msg) = unsafe { from_model(handle) }
            .map(|s| s.core.last_error())
            .unwrap_or_default();
        copy_utf8(&msg, out_buf, out_len)
    }));
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

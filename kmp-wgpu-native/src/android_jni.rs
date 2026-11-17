//! Android JNI：从 `android.view.Surface` 建 Session，供 Kotlin 外泵 tick。
//! Headless Compute（v1.1）句柄与 Surface Session **分表**，勿混用 tick/setCustomFs。

use crate::compute_readback::HeadlessCompute;
use crate::fullscreen_pass::FullscreenSessionCore;
use jni::objects::{JClass, JFloatArray, JObject, JString, JValue, ReleaseMode};
use jni::sys::{jboolean, jfloat, jint, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::NonNull;
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

fn log_e(env: &mut JNIEnv, msg: &str) {
    let Ok(tag) = env.new_string("kmp-wgpu-native") else {
        return;
    };
    let Ok(text) = env.new_string(msg) else {
        return;
    };
    let _ = env.call_static_method(
        "android/util/Log",
        "e",
        "(Ljava/lang/String;Ljava/lang/String;)I",
        &[JValue::Object(&tag), JValue::Object(&text)],
    );
}

struct AndroidWindowPtr(*mut ndk_sys::ANativeWindow);

// Safety: ANativeWindow 与 create/destroy 配对；Session 仅在 Holm 渲染线程使用。
unsafe impl Send for AndroidWindowPtr {}
unsafe impl Sync for AndroidWindowPtr {}

/// wgpu 30：`InstanceDescriptor::display` 与 `create_surface` 都需要 DisplayHandle。
/// Android 无独立 display 连接，用空的 `AndroidDisplayHandle` 作平台标记（须与 surface 侧一致）。
#[derive(Debug)]
struct AndroidDisplayOwner;

impl HasDisplayHandle for AndroidDisplayOwner {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Android(AndroidDisplayHandle::new()))
        })
    }
}

struct AndroidSurfaceTarget {
    window: AndroidWindowPtr,
}

impl HasWindowHandle for AndroidSurfaceTarget {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let ptr = NonNull::new(self.window.0 as *mut _).ok_or(HandleError::Unavailable)?;
        let handle = AndroidNdkWindowHandle::new(ptr);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(handle)) })
    }
}

impl HasDisplayHandle for AndroidSurfaceTarget {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let handle = AndroidDisplayHandle::new();
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Android(handle)) })
    }
}

struct NativeSession {
    core: FullscreenSessionCore,
    window: AndroidWindowPtr,
}

fn box_session(session: NativeSession) -> jlong {
    Box::into_raw(Box::new(session)) as jlong
}

unsafe fn from_handle<'a>(handle: jlong) -> Option<&'a mut NativeSession> {
    if handle == 0 {
        return None;
    }
    Some(&mut *(handle as *mut NativeSession))
}

fn release_window(ptr: *mut ndk_sys::ANativeWindow) {
    if !ptr.is_null() {
        unsafe { ndk_sys::ANativeWindow_release(ptr) };
    }
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeAbiVersion(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    crate::abi_version_inner() as jint
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeHasWgpu(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    crate::has_wgpu_inner() as jint
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    surface: JObject,
    width: jint,
    height: jint,
) -> jlong {
    if surface.is_null() {
        set_last_create_error("surface is null");
        log_e(&mut env, "nativeCreate: surface is null");
        return 0;
    }
    // fromSurface already acquires; release once in destroy
    let window_ptr =
        unsafe { ndk_sys::ANativeWindow_fromSurface(env.get_raw(), surface.as_raw()) };
    if window_ptr.is_null() {
        set_last_create_error("ANativeWindow_fromSurface returned null");
        log_e(&mut env, "nativeCreate: ANativeWindow_fromSurface null");
        return 0;
    }

    let w = width.max(1) as u32;
    let h = height.max(1) as u32;
    let target = AndroidSurfaceTarget {
        window: AndroidWindowPtr(window_ptr),
    };

    // wgpu 30：Android 先走 Vulkan（与 Ohos 对齐）；GL 作回退。勿静默 GLES 冒充成功。
    // from_window 会把 display 置 None → MissingDisplayHandle；须 Instance.display + from_display_and_window。
    let created = catch_unwind(AssertUnwindSafe(|| {
        let mut desc =
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(AndroidDisplayOwner));
        desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
        let instance = wgpu::Instance::new(desc);
        let target_unsafe =
            unsafe { wgpu::SurfaceTargetUnsafe::from_display_and_window(&target, &target) }
                .map_err(|e| format!("surface_target: {e}"))?;
        let surface = unsafe { instance.create_surface_unsafe(target_unsafe) }
            .map_err(|e| format!("create_surface: {e}"))?;
        let core = pollster::block_on(FullscreenSessionCore::create_with_surface(
            instance, surface, w, h, false,
        ))?;
        Ok::<FullscreenSessionCore, String>(core)
    }));

    match created {
        Ok(Ok(core)) => {
            clear_last_create_error();
            box_session(NativeSession {
                core,
                window: AndroidWindowPtr(window_ptr),
            })
        }
        Ok(Err(e)) => {
            set_last_create_error(&e);
            log_e(&mut env, &format!("nativeCreate failed: {e}"));
            release_window(window_ptr);
            0
        }
        Err(panic) => {
            let msg = format!("nativeCreate panic: {panic:?}");
            set_last_create_error(&msg);
            log_e(&mut env, &msg);
            release_window(window_ptr);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeLastCreateError(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let msg = LAST_CREATE_ERROR
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    env.new_string(msg)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    let boxed = unsafe { Box::from_raw(handle as *mut NativeSession) };
    let ptr = boxed.window.0;
    drop(boxed);
    release_window(ptr);
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeResize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
) {
    if let Some(s) = unsafe { from_handle(handle) } {
        s.core.resize(width.max(1) as u32, height.max(1) as u32);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeSetFragmentShader(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    fs: JString,
) -> jboolean {
    let Some(s) = (unsafe { from_handle(handle) }) else {
        return JNI_FALSE;
    };
    let Ok(src) = env.get_string(&fs) else {
        return JNI_FALSE;
    };
    match s.core.set_fragment_shader(&src.to_string_lossy()) {
        Ok(()) => JNI_TRUE,
        Err(_) => JNI_FALSE,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeSetPlaying(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    playing: jboolean,
) {
    if let Some(s) = unsafe { from_handle(handle) } {
        s.core.set_playing(playing != JNI_FALSE);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeTick(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    dt_secs: jfloat,
) -> jboolean {
    let Some(s) = (unsafe { from_handle(handle) }) else {
        return JNI_FALSE;
    };
    match s.core.tick(dt_secs) {
        Ok(()) => JNI_TRUE,
        Err(_) => JNI_FALSE,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeBackendId(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let id = unsafe { from_handle(handle) }
        .map(|s| s.core.backend_id)
        .unwrap_or("wgpu-blocked");
    env.new_string(id)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeLastErrorCode(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let (code, _) = unsafe { from_handle(handle) }
        .map(|s| s.core.last_error())
        .unwrap_or_default();
    env.new_string(code)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeLastErrorMessage(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let (_, msg) = unsafe { from_handle(handle) }
        .map(|s| s.core.last_error())
        .unwrap_or_default();
    env.new_string(msg)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// 返回 String[]：code, message, code, message, …
#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeDrainErrors<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jni::sys::jobjectArray {
    let Some(s) = (unsafe { from_handle(handle) }) else {
        return env
            .new_object_array(0, "java/lang/String", JObject::null())
            .map(|a| a.into_raw())
            .unwrap_or(std::ptr::null_mut());
    };
    let errs = s.core.drain_errors();
    let len = (errs.len() * 2) as i32;
    let Ok(arr) = env.new_object_array(len, "java/lang/String", JObject::null()) else {
        return env
            .new_object_array(0, "java/lang/String", JObject::null())
            .map(|a| a.into_raw())
            .unwrap_or(std::ptr::null_mut());
    };
    for (i, (code, msg)) in errs.into_iter().enumerate() {
        let Ok(jc) = env.new_string(&code) else {
            continue;
        };
        let Ok(jm) = env.new_string(&msg) else {
            continue;
        };
        let _ = env.set_object_array_element(&arr, (i * 2) as i32, jc);
        let _ = env.set_object_array_element(&arr, (i * 2 + 1) as i32, jm);
    }
    arr.into_raw()
}

// --- Headless Compute（v1.1）· 与 NativeSession 分表 ---

fn box_headless(session: HeadlessCompute) -> jlong {
    Box::into_raw(Box::new(session)) as jlong
}

unsafe fn from_headless<'a>(handle: jlong) -> Option<&'a mut HeadlessCompute> {
    if handle == 0 {
        return None;
    }
    Some(&mut *(handle as *mut HeadlessCompute))
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeCreateHeadless(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    let created = catch_unwind(AssertUnwindSafe(|| {
        pollster::block_on(HeadlessCompute::create(false))
    }));
    match created {
        Ok(Ok(session)) => {
            clear_last_create_error();
            box_headless(session)
        }
        Ok(Err(e)) => {
            set_last_create_error(&e);
            log_e(&mut env, &format!("nativeCreateHeadless failed: {e}"));
            0
        }
        Err(panic) => {
            let msg = format!("nativeCreateHeadless panic: {panic:?}");
            set_last_create_error(&msg);
            log_e(&mut env, &msg);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeDestroyHeadless(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    let _ = unsafe { Box::from_raw(handle as *mut HeadlessCompute) };
}

#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeHeadlessBackendId(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let id = unsafe { from_headless(handle) }
        .map(|s| s.backend_id)
        .unwrap_or("wgpu-blocked");
    env.new_string(id)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// 粗块：WGSL compute + f32[] 回读。失败返回 null，文案进 lastCreateError。
#[no_mangle]
pub extern "system" fn Java_com_jlj_kmp_wgpu_AndroidWgpuPlatform_nativeComputeReadback<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    wgsl: JString<'local>,
    input: JFloatArray<'local>,
    workgroup_count_x: jint,
) -> jni::sys::jfloatArray {
    let Some(session) = (unsafe { from_headless(handle) }) else {
        set_last_create_error("nativeComputeReadback: bad headless handle");
        return std::ptr::null_mut();
    };
    let Ok(src) = env.get_string(&wgsl) else {
        set_last_create_error("nativeComputeReadback: bad wgsl");
        return std::ptr::null_mut();
    };
    let Ok(elems) = env.get_array_length(&input) else {
        set_last_create_error("nativeComputeReadback: bad input length");
        return std::ptr::null_mut();
    };
    if elems <= 0 {
        set_last_create_error("nativeComputeReadback: empty input");
        return std::ptr::null_mut();
    }
    let Ok(auto) = (unsafe { env.get_array_elements(&input, ReleaseMode::NoCopyBack) }) else {
        set_last_create_error("nativeComputeReadback: get_array_elements failed");
        return std::ptr::null_mut();
    };
    let input_vec: Vec<f32> = auto.iter().copied().collect();
    drop(auto);

    let wg = workgroup_count_x.max(1) as u32;
    let result = catch_unwind(AssertUnwindSafe(|| {
        pollster::block_on(session.run_f32(&src.to_string_lossy(), &input_vec, wg))
    }));
    match result {
        Ok(Ok(out)) => {
            clear_last_create_error();
            let Ok(arr) = env.new_float_array(out.len() as i32) else {
                set_last_create_error("nativeComputeReadback: new_float_array failed");
                return std::ptr::null_mut();
            };
            if env.set_float_array_region(&arr, 0, &out).is_err() {
                set_last_create_error("nativeComputeReadback: set_float_array_region failed");
                return std::ptr::null_mut();
            }
            arr.into_raw()
        }
        Ok(Err(e)) => {
            set_last_create_error(&e);
            log_e(&mut env, &format!("nativeComputeReadback failed: {e}"));
            std::ptr::null_mut()
        }
        Err(panic) => {
            let msg = format!("nativeComputeReadback panic: {panic:?}");
            set_last_create_error(&msg);
            log_e(&mut env, &msg);
            std::ptr::null_mut()
        }
    }
}

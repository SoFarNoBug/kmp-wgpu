//! **kmp_wgpu_native**（ADR 0013 / 0014 / 0015）— wgpu 原语 + FullscreenPass。
//!
//! 产物 / 符号：`kmp_wgpu_native` / `kmp_wgpu_*`。
//! 内部：[`primitives`] + [`fullscreen_pass`]；JNI / C / wasm 薄封装。

#![allow(dead_code)]

#[cfg(feature = "wgpu_backend")]
mod shaders;

#[cfg(feature = "wgpu_backend")]
mod error_bridge;

#[cfg(feature = "wgpu_backend")]
mod primitives;

#[cfg(feature = "wgpu_backend")]
mod fullscreen_pass;

#[cfg(feature = "wgpu_backend")]
mod model_pass;

#[cfg(feature = "wgpu_backend")]
mod compute_readback;

#[cfg(all(target_arch = "wasm32", feature = "wgpu_backend"))]
mod session;

#[cfg(all(target_arch = "wasm32", feature = "wgpu_backend"))]
pub use session::KmpWgpuSession;

#[cfg(all(target_os = "android", feature = "wgpu_backend"))]
mod android_jni;

#[cfg(all(target_os = "ios", feature = "wgpu_backend"))]
mod ios_c_api;

#[cfg(all(target_env = "ohos", feature = "wgpu_backend"))]
mod ohos_c_api;

pub const BACKEND_ID: &str = "wgpu-m0";

pub(crate) fn abi_version_inner() -> u32 {
    1
}

pub(crate) fn has_wgpu_inner() -> u32 {
    #[cfg(feature = "wgpu_backend")]
    {
        1
    }
    #[cfg(not(feature = "wgpu_backend"))]
    {
        0
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn kmp_wgpu_abi_version() -> u32 {
    abi_version_inner()
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn kmp_wgpu_has_wgpu() -> u32 {
    has_wgpu_inner()
}

/// 遗留 stub：通面请走 Android JNI / iOS C Session / H5 KmpWgpuSession。
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn kmp_wgpu_m0_clear(_width: u32, _height: u32) -> i32 {
    -1
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn kmp_wgpu_m0_triangle(_width: u32, _height: u32) -> i32 {
    -1
}

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    use super::{abi_version_inner, has_wgpu_inner};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn kmp_wgpu_abi_version() -> u32 {
        abi_version_inner()
    }

    #[wasm_bindgen]
    pub fn kmp_wgpu_has_wgpu() -> u32 {
        has_wgpu_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_version_is_one() {
        assert_eq!(abi_version_inner(), 1);
    }
}

//! kmp-wgpu 原语边界（ADR 0013 / 0015）。
//!
//! Device / Queue / Surface 由 [`crate::fullscreen_pass`] 持有；
//! Present 产品面：`set_fragment_shader` / tick / playing（无 liquid/scene/ClearTri）。

#![allow(unused_imports)]

#[cfg(feature = "wgpu_backend")]
pub use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

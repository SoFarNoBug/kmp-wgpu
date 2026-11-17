//! Fullscreen VS + FrameUniforms；业务片元由宿主 set_fragment_shader 提供。

pub const FULLSCREEN_VS: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f( 3.0, -1.0),
        vec2f(-1.0,  3.0),
    );
    var out: VsOut;
    out.pos = vec4f(p[vi], 0.0, 1.0);
    out.uv = p[vi] * 0.5 + vec2f(0.5, 0.5);
    return out;
}
"#;

/// time + pads + resolution（ADR 0015：已删 liquid_*）
pub const FRAME_UNIFORMS_WGSL: &str = r#"
struct FrameUniforms {
    time: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    resolution: vec2f,
    _pad3: vec2f,
}

@group(0) @binding(0) var<uniform> u: FrameUniforms;
"#;

pub fn module_source(fs: &str) -> String {
    format!("{}\n{}\n{}", FULLSCREEN_VS, FRAME_UNIFORMS_WGSL, fs)
}

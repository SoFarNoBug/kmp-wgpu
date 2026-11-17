package com.jlj.kmp.wgpu.sample

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: 无 Holm sample · 纯色片元（FullscreenPass 冒烟）。
 */
object SolidFragment {
    const val WGSL: String = """
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
    let t = u.time;
    let uv = in.uv;
    return vec4f(uv.x, uv.y, 0.5 + 0.5 * sin(t), 1.0);
}
""".trimIndent()
}

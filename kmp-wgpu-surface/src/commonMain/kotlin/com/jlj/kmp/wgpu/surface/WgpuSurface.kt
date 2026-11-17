package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: kmp-wgpu-surface · 可选 Present（ADR 0013）。默认不在 kmp-wgpu core。
 */

/** Surface 配置（近 wgpu SurfaceConfiguration；v1 仅宽高）。 */
data class SurfaceConfiguration(
    val width: Int,
    val height: Int,
)

/**
 * 呈现表面句柄。v1 与 Device 同生同灭（createDevice 时绑定窗口/canvas）。
 */
class GpuSurface(
    val device: Device,
    initialConfig: SurfaceConfiguration,
) {
    private var _config: SurfaceConfiguration = initialConfig
    val configuration: SurfaceConfiguration get() = _config

    fun configure(config: SurfaceConfiguration) {
        _config = SurfaceConfiguration(
            width = config.width.coerceAtLeast(1),
            height = config.height.coerceAtLeast(1),
        )
        resizePlatform(device, _config.width, _config.height)
    }
}

internal expect fun resizePlatform(device: Device, width: Int, height: Int)

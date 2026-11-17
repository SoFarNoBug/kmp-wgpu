package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: 通用全屏片元 Present（ADR 0015）。无片元时 tick 只清屏；无 liquid/scene。
 */
class FullscreenPass(
    val gpuSurface: GpuSurface,
) {
    val device: Device get() = gpuSurface.device

    fun resize(width: Int, height: Int) {
        gpuSurface.configure(
            SurfaceConfiguration(
                width = width.coerceAtLeast(1),
                height = height.coerceAtLeast(1),
            ),
        )
    }

    fun setFragmentShader(wgslFs: String): Boolean =
        platformSetFragmentShader(device, wgslFs)

    fun setPlaying(playing: Boolean) {
        platformSetPlaying(device, playing)
    }

    fun tick(dtSecs: Float): Boolean = platformTick(device, dtSecs)

    fun backendId(): String = platformBackendId(device)

    fun lastErrorCode(): String = platformLastErrorCode(device)

    fun lastErrorMessage(): String = platformLastErrorMessage(device)

    fun drainErrors(): List<Pair<String, String>> = platformDrainErrors(device)

    fun destroy() {
        device.destroy()
    }
}

internal expect fun platformSetFragmentShader(device: Device, wgslFs: String): Boolean

internal expect fun platformSetPlaying(device: Device, playing: Boolean)

internal expect fun platformTick(device: Device, dtSecs: Float): Boolean

internal expect fun platformBackendId(device: Device): String

internal expect fun platformLastErrorCode(device: Device): String

internal expect fun platformLastErrorMessage(device: Device): String

internal expect fun platformDrainErrors(device: Device): List<Pair<String, String>>

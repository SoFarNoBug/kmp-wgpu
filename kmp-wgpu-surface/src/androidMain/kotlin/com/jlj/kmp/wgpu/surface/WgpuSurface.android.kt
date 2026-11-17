package com.jlj.kmp.wgpu.surface

import android.view.Surface
import com.jlj.kmp.wgpu.Device
import com.jlj.kmp.wgpu.WgpuRuntime
import com.jlj.kmp.wgpu.androidPlatform

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: Android Surface → Device + GpuSurface。
 */

object AndroidSurfaces {
    fun create(
        surface: Surface,
        width: Int,
        height: Int,
    ): GpuSurface? {
        WgpuRuntime.createInstance()
        val device = androidPlatform().createDeviceFromSurface(surface, width, height) ?: return null
        return GpuSurface(
            device = device,
            initialConfig = SurfaceConfiguration(width.coerceAtLeast(1), height.coerceAtLeast(1)),
        )
    }

    /** ADR 0015：直接产出 FullscreenPass。 */
    fun createFullscreenPass(
        surface: Surface,
        width: Int,
        height: Int,
    ): FullscreenPass? {
        val gpu = create(surface, width, height) ?: return null
        return FullscreenPass(gpu)
    }

    fun lastCreateError(): String = androidPlatform().lastCreateError()
}

internal actual fun resizePlatform(device: Device, width: Int, height: Int) {
    androidPlatform().resize(device.handle, width, height)
}

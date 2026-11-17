@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device
import com.jlj.kmp.wgpu.WgpuRuntime
import com.jlj.kmp.wgpu.ohosPlatform
import kotlinx.cinterop.COpaquePointer

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: Ohos OHNativeWindow* → Device + GpuSurface。
 */

object OhosSurfaces {
    fun create(
        nativeWindow: COpaquePointer?,
        width: Int,
        height: Int,
    ): GpuSurface? {
        WgpuRuntime.createInstance()
        val device = ohosPlatform().createDeviceFromNativeWindow(nativeWindow, width, height) ?: return null
        return GpuSurface(
            device = device,
            initialConfig = SurfaceConfiguration(width.coerceAtLeast(1), height.coerceAtLeast(1)),
        )
    }

    fun lastCreateError(): String = ohosPlatform().lastCreateError()
}

internal actual fun resizePlatform(device: Device, width: Int, height: Int) {
    ohosPlatform().resize(device.handle, width, height)
}

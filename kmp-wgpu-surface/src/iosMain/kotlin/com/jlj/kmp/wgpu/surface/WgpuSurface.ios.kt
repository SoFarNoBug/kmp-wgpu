@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device
import com.jlj.kmp.wgpu.WgpuRuntime
import com.jlj.kmp.wgpu.iosPlatform
import kotlinx.cinterop.COpaquePointer

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: iOS CAMetalLayer* → Device + GpuSurface。
 */

object IosSurfaces {
    fun create(
        metalLayer: COpaquePointer?,
        width: Int,
        height: Int,
    ): GpuSurface? {
        WgpuRuntime.createInstance()
        val device = iosPlatform().createDeviceFromMetalLayer(metalLayer, width, height) ?: return null
        return GpuSurface(
            device = device,
            initialConfig = SurfaceConfiguration(width.coerceAtLeast(1), height.coerceAtLeast(1)),
        )
    }

    fun lastCreateError(): String = iosPlatform().lastCreateError()
}

internal actual fun resizePlatform(device: Device, width: Int, height: Int) {
    iosPlatform().resize(device.handle, width, height)
}

package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device
import com.jlj.kmp.wgpu.iosPlatform

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: iOS FullscreenPass → IosWgpuPlatform cinterop。
 */

internal actual fun platformSetFragmentShader(device: Device, wgslFs: String): Boolean =
    iosPlatform().setFragmentShader(device.handle, wgslFs)

internal actual fun platformSetPlaying(device: Device, playing: Boolean) {
    iosPlatform().setPlaying(device.handle, playing)
}

internal actual fun platformTick(device: Device, dtSecs: Float): Boolean =
    iosPlatform().tick(device.handle, dtSecs)

internal actual fun platformBackendId(device: Device): String =
    iosPlatform().backendId(device.handle)

internal actual fun platformLastErrorCode(device: Device): String =
    iosPlatform().lastErrorCode(device.handle)

internal actual fun platformLastErrorMessage(device: Device): String =
    iosPlatform().lastErrorMessage(device.handle)

internal actual fun platformDrainErrors(device: Device): List<Pair<String, String>> =
    iosPlatform().drainErrors(device.handle)

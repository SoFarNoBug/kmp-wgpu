package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device
import com.jlj.kmp.wgpu.ohosPlatform

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: Ohos FullscreenPass → OhosWgpuPlatform cinterop。
 */

internal actual fun platformSetFragmentShader(device: Device, wgslFs: String): Boolean =
    ohosPlatform().setFragmentShader(device.handle, wgslFs)

internal actual fun platformSetPlaying(device: Device, playing: Boolean) {
    ohosPlatform().setPlaying(device.handle, playing)
}

internal actual fun platformTick(device: Device, dtSecs: Float): Boolean =
    ohosPlatform().tick(device.handle, dtSecs)

internal actual fun platformBackendId(device: Device): String =
    ohosPlatform().backendId(device.handle)

internal actual fun platformLastErrorCode(device: Device): String =
    ohosPlatform().lastErrorCode(device.handle)

internal actual fun platformLastErrorMessage(device: Device): String =
    ohosPlatform().lastErrorMessage(device.handle)

internal actual fun platformDrainErrors(device: Device): List<Pair<String, String>> =
    ohosPlatform().drainErrors(device.handle)

package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device
import com.jlj.kmp.wgpu.androidPlatform

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: Android FullscreenPass → AndroidWgpuPlatform JNI。
 */

internal actual fun platformSetFragmentShader(device: Device, wgslFs: String): Boolean =
    androidPlatform().setFragmentShader(device.handle, wgslFs)

internal actual fun platformSetPlaying(device: Device, playing: Boolean) {
    androidPlatform().setPlaying(device.handle, playing)
}

internal actual fun platformTick(device: Device, dtSecs: Float): Boolean =
    androidPlatform().tick(device.handle, dtSecs)

internal actual fun platformBackendId(device: Device): String =
    androidPlatform().backendId(device.handle)

internal actual fun platformLastErrorCode(device: Device): String =
    androidPlatform().lastErrorCode(device.handle)

internal actual fun platformLastErrorMessage(device: Device): String =
    androidPlatform().lastErrorMessage(device.handle)

internal actual fun platformDrainErrors(device: Device): List<Pair<String, String>> =
    androidPlatform().drainErrors(device.handle)

package com.jlj.kmp.wgpu

import java.util.concurrent.Executors

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: Android headless Compute + 回读 → JNI（与 Surface Session 分表）。
 */

private val headlessExecutor = Executors.newSingleThreadExecutor { r ->
    Thread(r, "kmp-wgpu-headless").apply { isDaemon = true }
}

actual fun createHeadlessDevice(): HeadlessDevice? {
    if (!WgpuRuntime.isAvailable()) return null
    val handle = AndroidWgpuPlatform.createHeadless()
    if (handle == 0L) return null
    return HeadlessDevice(handle) { h ->
        AndroidWgpuPlatform.destroyHeadless(h as Long)
    }
}

actual fun createHeadlessDeviceAsync(onDone: (Result<HeadlessDevice>) -> Unit) {
    headlessExecutor.execute {
        try {
            val device = createHeadlessDevice()
            if (device == null) {
                val err = AndroidWgpuPlatform.lastCreateError().ifBlank {
                    WgpuRuntime.blockedMessage()
                }
                onDone(Result.failure(IllegalStateException(err)))
            } else {
                onDone(Result.success(device))
            }
        } catch (t: Throwable) {
            onDone(Result.failure(t))
        }
    }
}

actual fun HeadlessDevice.computeReadback(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
): Result<FloatArray> {
    if (!isAlive) {
        return Result.failure(IllegalStateException("headless device destroyed"))
    }
    val handle = this.handle as Long
    if (handle == 0L) {
        return Result.failure(IllegalStateException("headless handle is 0"))
    }
    if (input.isEmpty()) {
        return Result.failure(IllegalArgumentException("empty input"))
    }
    val out = AndroidWgpuPlatform.computeReadback(
        handle,
        shaderWgsl,
        input,
        workgroupCountX.coerceAtLeast(1),
    )
    return if (out == null) {
        Result.failure(
            IllegalStateException(
                AndroidWgpuPlatform.lastCreateError().ifBlank { "computeReadback failed" },
            ),
        )
    } else {
        Result.success(out)
    }
}

actual fun HeadlessDevice.computeReadbackAsync(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
    onDone: (Result<FloatArray>) -> Unit,
) {
    val wgslCopy = shaderWgsl
    val inputCopy = input.copyOf()
    val wg = workgroupCountX.coerceAtLeast(1)
    headlessExecutor.execute {
        onDone(computeReadback(wgslCopy, inputCopy, wg))
    }
}

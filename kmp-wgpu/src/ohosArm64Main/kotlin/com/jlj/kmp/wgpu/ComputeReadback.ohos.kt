@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.jlj.kmp.wgpu

import kmp.wgpu.native.cinterop.kmp_wgpu_headless_backend_id
import kmp.wgpu.native.cinterop.kmp_wgpu_headless_compute_readback
import kmp.wgpu.native.cinterop.kmp_wgpu_headless_create
import kmp.wgpu.native.cinterop.kmp_wgpu_headless_destroy
import kmp.wgpu.native.cinterop.kmp_wgpu_last_create_error
import kotlinx.cinterop.ByteVar
import kotlinx.cinterop.allocArray
import kotlinx.cinterop.get
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.toKString
import kotlinx.cinterop.usePinned
import kotlinx.cinterop.addressOf

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: Ohos headless Compute + 回读 → C ABI（与 Present Session 分表）。
 */

private fun lastCreateError(): String =
    memScoped {
        val cap = 512u
        val buf = allocArray<ByteVar>(cap.toInt())
        runCatching { kmp_wgpu_last_create_error(buf, cap) }.getOrDefault(0u)
        if (buf[0].toInt() == 0) "" else buf.toKString()
    }

actual fun createHeadlessDevice(): HeadlessDevice? {
    if (!WgpuRuntime.isAvailable()) return null
    val handle = runCatching { kmp_wgpu_headless_create().toLong() }.getOrDefault(0L)
    if (handle == 0L) return null
    return HeadlessDevice(handle) { h ->
        runCatching { kmp_wgpu_headless_destroy(h.toULong()) }
    }
}

actual fun createHeadlessDeviceAsync(onDone: (Result<HeadlessDevice>) -> Unit) {
    // 短样本：同步创建；Demo 在 UI 线程调用可接受
    try {
        val device = createHeadlessDevice()
        if (device == null) {
            val err = lastCreateError().ifBlank { WgpuRuntime.blockedMessage() }
            onDone(Result.failure(IllegalStateException(err)))
        } else {
            onDone(Result.success(device))
        }
    } catch (t: Throwable) {
        onDone(Result.failure(t))
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
    val handle = this.handle
    if (handle == 0L) {
        return Result.failure(IllegalStateException("headless handle is 0"))
    }
    if (input.isEmpty()) {
        return Result.failure(IllegalArgumentException("empty input"))
    }
    val out = FloatArray(input.size)
    val n = runCatching {
        input.usePinned { inPin ->
            out.usePinned { outPin ->
                kmp_wgpu_headless_compute_readback(
                    handle.toULong(),
                    shaderWgsl,
                    inPin.addressOf(0),
                    input.size.toUInt(),
                    workgroupCountX.coerceAtLeast(1).toUInt(),
                    outPin.addressOf(0),
                    out.size.toUInt(),
                ).toInt()
            }
        }
    }.getOrDefault(0)
    return if (n <= 0) {
        Result.failure(
            IllegalStateException(lastCreateError().ifBlank { "computeReadback failed" }),
        )
    } else if (n == out.size) {
        Result.success(out)
    } else {
        Result.success(out.copyOf(n))
    }
}

actual fun HeadlessDevice.computeReadbackAsync(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
    onDone: (Result<FloatArray>) -> Unit,
) {
    onDone(computeReadback(shaderWgsl, input.copyOf(), workgroupCountX.coerceAtLeast(1)))
}

/** 调试用：headless backend id。 */
fun HeadlessDevice.ohosBackendId(): String {
    if (handle == 0L) return "wgpu-blocked"
    return memScoped {
        val cap = 128u
        val buf = allocArray<ByteVar>(cap.toInt())
        runCatching {
            kmp_wgpu_headless_backend_id(handle.toULong(), buf, cap)
        }.getOrDefault(0u)
        if (buf[0].toInt() == 0) "wgpu-blocked" else buf.toKString()
    }
}

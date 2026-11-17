@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.jlj.kmp.wgpu

import kmp.wgpu.native.cinterop.kmp_wgpu_headless_backend_id
import kmp.wgpu.native.cinterop.kmp_wgpu_headless_compute_readback
import kmp.wgpu.native.cinterop.kmp_wgpu_headless_create
import kmp.wgpu.native.cinterop.kmp_wgpu_headless_destroy
import kmp.wgpu.native.cinterop.kmp_wgpu_last_create_error
import kotlinx.cinterop.ByteVar
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.allocArray
import kotlinx.cinterop.get
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.toKString
import kotlinx.cinterop.usePinned
import platform.darwin.dispatch_async
import platform.darwin.dispatch_get_main_queue
import platform.darwin.dispatch_queue_create

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: iOS headless Compute + 回读 → C ABI（Metal；与 Present Session 分表）。
 */

/** 单串行队列：create / compute / destroy 同序，避免跨线程碰同一 Device。attr=null = SERIAL。 */
private val headlessQueue =
    dispatch_queue_create("com.jlj.kmp.wgpu.headless", null)

private fun lastCreateError(): String =
    memScoped {
        val cap = 512u
        val buf = allocArray<ByteVar>(cap.toInt())
        runCatching { kmp_wgpu_last_create_error(buf, cap) }.getOrDefault(0u)
        if (buf[0].toInt() == 0) "" else buf.toKString()
    }

private fun runOnHeadless(block: () -> Unit) {
    dispatch_async(headlessQueue, block)
}

private fun runOnMain(block: () -> Unit) {
    dispatch_async(dispatch_get_main_queue(), block)
}

actual fun createHeadlessDevice(): HeadlessDevice? {
    if (!WgpuRuntime.isAvailable()) return null
    val handle = runCatching { kmp_wgpu_headless_create().toLong() }.getOrDefault(0L)
    if (handle == 0L) return null
    return HeadlessDevice(handle) { h ->
        // destroy 可能从主线程触发；落到串行队列与 create/compute 同序
        runOnHeadless {
            runCatching { kmp_wgpu_headless_destroy(h.toULong()) }
        }
    }
}

actual fun createHeadlessDeviceAsync(onDone: (Result<HeadlessDevice>) -> Unit) {
    runOnHeadless {
        try {
            val device = createHeadlessDevice()
            val result = if (device == null) {
                val err = lastCreateError().ifBlank { WgpuRuntime.blockedMessage() }
                Result.failure(IllegalStateException(err))
            } else {
                Result.success(device)
            }
            runOnMain { onDone(result) }
        } catch (t: Throwable) {
            runOnMain { onDone(Result.failure(t)) }
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
    val wgslCopy = shaderWgsl
    val inputCopy = input.copyOf()
    val wg = workgroupCountX.coerceAtLeast(1)
    runOnHeadless {
        val result = computeReadback(wgslCopy, inputCopy, wg)
        runOnMain { onDone(result) }
    }
}

/** 调试用：headless backend id。 */
fun HeadlessDevice.iosBackendId(): String {
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

@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.jlj.kmp.wgpu

import kmp.wgpu.native.cinterop.kmp_wgpu_abi_version
import kmp.wgpu.native.cinterop.kmp_wgpu_has_wgpu
import kmp.wgpu.native.cinterop.kmp_wgpu_last_create_error
import kmp.wgpu.native.cinterop.kmp_wgpu_session_backend_id
import kmp.wgpu.native.cinterop.kmp_wgpu_session_create_metal
import kmp.wgpu.native.cinterop.kmp_wgpu_session_destroy
import kmp.wgpu.native.cinterop.kmp_wgpu_session_last_error_code
import kmp.wgpu.native.cinterop.kmp_wgpu_session_last_error_message
import kmp.wgpu.native.cinterop.kmp_wgpu_session_resize
import kmp.wgpu.native.cinterop.kmp_wgpu_session_set_fragment_shader
import kmp.wgpu.native.cinterop.kmp_wgpu_session_set_playing
import kmp.wgpu.native.cinterop.kmp_wgpu_session_tick
import kotlin.concurrent.Volatile
import kotlinx.cinterop.ByteVar
import kotlinx.cinterop.COpaquePointer
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.allocArray
import kotlinx.cinterop.get
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.toKString

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: iOS kmp-wgpu 绑定；cinterop → KmpWgpuNative.xcframework（ADR 0009 C ABI）。
 * 粗粒度能力块：create / tick / resize / destroy；无逐 draw FFI。
 * App 仍须链 KmpWgpuNative.xcframework（与今日 Xcode 直链一致）。
 */

actual typealias WgpuHandle = Long

object IosWgpuPlatform : WgpuPlatform {
    @Volatile
    private var _probed: Boolean? = null

    override fun isLibraryLoaded(): Boolean {
        val cached = _probed
        if (cached != null) return cached
        val ok = runCatching {
            kmp_wgpu_abi_version()
            true
        }.getOrDefault(false)
        _probed = ok
        return ok
    }

    override fun isAvailable(): Boolean =
        isLibraryLoaded() && runCatching { kmp_wgpu_has_wgpu() }.getOrDefault(0u) == 1u

    override fun blockedMessage(): String {
        return if (!isLibraryLoaded()) {
            "kmp-wgpu BLOCKED: KmpWgpuNative.xcframework missing (run kmp-wgpu-native/build-ios.sh). No silent non-wgpu stack."
        } else {
            "kmp-wgpu BLOCKED: kmp_wgpu_native linked but has_wgpu=0. No silent GLES stand-in."
        }
    }

    override fun destroyDevice(handle: WgpuHandle) {
        if (handle == 0L || !isLibraryLoaded()) return
        runCatching { kmp_wgpu_session_destroy(handle.toULong()) }
    }

    fun createDeviceFromMetalLayer(layer: COpaquePointer?, width: Int, height: Int): Device? {
        if (layer == null || !isLibraryLoaded()) return null
        val handle = runCatching {
            kmp_wgpu_session_create_metal(
                layer,
                width.coerceAtLeast(1).toUInt(),
                height.coerceAtLeast(1).toUInt(),
            ).toLong()
        }.getOrDefault(0L)
        if (handle == 0L) return null
        return Device(handle, this)
    }

    fun abiVersion(): Int {
        if (!isLibraryLoaded()) return 0
        return runCatching { kmp_wgpu_abi_version().toInt() }.getOrDefault(0)
    }

    fun lastCreateError(): String {
        if (!isLibraryLoaded()) return ""
        return readCStringBuf { buf, len ->
            kmp_wgpu_last_create_error(buf, len)
        }
    }

    fun resize(handle: WgpuHandle, width: Int, height: Int) {
        if (handle == 0L) return
        runCatching {
            kmp_wgpu_session_resize(
                handle.toULong(),
                width.coerceAtLeast(1).toUInt(),
                height.coerceAtLeast(1).toUInt(),
            )
        }
    }

    fun setFragmentShader(handle: WgpuHandle, fs: String): Boolean {
        if (handle == 0L || !isLibraryLoaded()) return false
        return runCatching {
            kmp_wgpu_session_set_fragment_shader(handle.toULong(), fs) == 1u
        }.getOrDefault(false)
    }

    fun setPlaying(handle: WgpuHandle, playing: Boolean) {
        if (handle == 0L || !isLibraryLoaded()) return
        runCatching {
            kmp_wgpu_session_set_playing(handle.toULong(), if (playing) 1u else 0u)
        }
    }

    fun tick(handle: WgpuHandle, dtSecs: Float): Boolean {
        if (handle == 0L) return false
        return runCatching {
            kmp_wgpu_session_tick(handle.toULong(), dtSecs) == 1u
        }.getOrDefault(false)
    }

    fun backendId(handle: WgpuHandle): String {
        if (handle == 0L) return "wgpu-blocked"
        return readCStringBuf { buf, len ->
            kmp_wgpu_session_backend_id(handle.toULong(), buf, len)
        }.ifBlank { "wgpu-m0" }
    }

    fun lastErrorCode(handle: WgpuHandle): String {
        if (handle == 0L) return ""
        return readCStringBuf { buf, len ->
            kmp_wgpu_session_last_error_code(handle.toULong(), buf, len)
        }
    }

    fun lastErrorMessage(handle: WgpuHandle): String {
        if (handle == 0L) return ""
        return readCStringBuf { buf, len ->
            kmp_wgpu_session_last_error_message(handle.toULong(), buf, len)
        }
    }

    /**
     * iOS C ABI 无 drain；用 last_error 合成最多一条（对齐壳侧原先单错误读法）。
     */
    fun drainErrors(handle: WgpuHandle): List<Pair<String, String>> {
        if (handle == 0L) return emptyList()
        val code = lastErrorCode(handle)
        val msg = lastErrorMessage(handle)
        if (code.isBlank() && msg.isBlank()) return emptyList()
        return listOf(code to msg)
    }

    private fun readCStringBuf(block: (buf: CPointer<ByteVar>, len: UInt) -> UInt): String =
        memScoped {
            val cap = 512u
            val buf = allocArray<ByteVar>(cap.toInt())
            runCatching { block(buf, cap) }.getOrDefault(0u)
            if (buf[0].toInt() == 0) return@memScoped ""
            buf.toKString()
        }
}

actual object WgpuRuntime {
    actual fun createInstance(): Instance = Instance(IosWgpuPlatform)

    actual fun isLibraryLoaded(): Boolean = IosWgpuPlatform.isLibraryLoaded()

    actual fun isAvailable(): Boolean = IosWgpuPlatform.isAvailable()

    actual fun blockedMessage(): String = IosWgpuPlatform.blockedMessage()
}

fun iosPlatform(): IosWgpuPlatform = IosWgpuPlatform

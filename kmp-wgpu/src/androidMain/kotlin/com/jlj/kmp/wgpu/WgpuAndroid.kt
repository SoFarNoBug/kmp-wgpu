package com.jlj.kmp.wgpu

import android.view.Surface

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: Android kmp-wgpu 绑定；JNI 进 libkmp_wgpu_native.so。
 * 粗粒度能力块：create / tick / resize / destroy；无逐 draw FFI。
 */

actual typealias WgpuHandle = Long

object AndroidWgpuPlatform : WgpuPlatform {
    @Volatile
    private var _libraryLoaded: Boolean? = null

    override fun isLibraryLoaded(): Boolean {
        val cached = _libraryLoaded
        if (cached != null) return cached
        val ok = runCatching {
            System.loadLibrary("kmp_wgpu_native")
            true
        }.getOrDefault(false)
        _libraryLoaded = ok
        return ok
    }

    override fun isAvailable(): Boolean = isLibraryLoaded() && runCatching { nativeHasWgpu() }.getOrDefault(0) == 1

    override fun blockedMessage(): String {
        return if (!isLibraryLoaded()) {
            "kmp-wgpu BLOCKED: libkmp_wgpu_native.so missing (run kmp-wgpu-native/build-android.ps1). No silent non-wgpu stack."
        } else {
            "kmp-wgpu BLOCKED: kmp_wgpu_native loaded but has_wgpu=0. No silent GLES stand-in."
        }
    }

    override fun destroyDevice(handle: WgpuHandle) {
        if (handle == 0L || !isLibraryLoaded()) return
        runCatching { nativeDestroy(handle) }
    }

    fun createDeviceFromSurface(surface: Surface, width: Int, height: Int): Device? {
        if (!isLibraryLoaded()) return null
        val handle = runCatching {
            nativeCreate(surface, width.coerceAtLeast(1), height.coerceAtLeast(1))
        }.getOrDefault(0L)
        if (handle == 0L) return null
        return Device(handle, this)
    }

    fun abiVersion(): Int {
        if (!isLibraryLoaded()) return 0
        return runCatching { nativeAbiVersion() }.getOrDefault(0)
    }

    fun lastCreateError(): String {
        if (!isLibraryLoaded()) return ""
        return runCatching { nativeLastCreateError() }.getOrDefault("")
    }

    fun resize(handle: WgpuHandle, width: Int, height: Int) {
        if (handle == 0L) return
        runCatching { nativeResize(handle, width, height) }
    }

    fun setFragmentShader(handle: WgpuHandle, fs: String): Boolean {
        if (handle == 0L) return false
        return runCatching { nativeSetFragmentShader(handle, fs) }.getOrDefault(false)
    }

    fun setPlaying(handle: WgpuHandle, playing: Boolean) {
        if (handle == 0L) return
        runCatching { nativeSetPlaying(handle, playing) }
    }

    fun tick(handle: WgpuHandle, dtSecs: Float): Boolean {
        if (handle == 0L) return false
        return runCatching { nativeTick(handle, dtSecs) }.getOrDefault(false)
    }

    fun backendId(handle: WgpuHandle): String {
        if (handle == 0L) return "wgpu-blocked"
        return runCatching { nativeBackendId(handle) }.getOrDefault("wgpu-m0")
    }

    fun lastErrorCode(handle: WgpuHandle): String {
        if (handle == 0L) return ""
        return runCatching { nativeLastErrorCode(handle) }.getOrDefault("")
    }

    fun lastErrorMessage(handle: WgpuHandle): String {
        if (handle == 0L) return ""
        return runCatching { nativeLastErrorMessage(handle) }.getOrDefault("")
    }

    fun drainErrors(handle: WgpuHandle): List<Pair<String, String>> {
        if (handle == 0L) return emptyList()
        val flat = runCatching { nativeDrainErrors(handle) }.getOrNull() ?: return emptyList()
        val out = ArrayList<Pair<String, String>>(flat.size / 2)
        var i = 0
        while (i + 1 < flat.size) {
            out.add(flat[i] to flat[i + 1])
            i += 2
        }
        return out
    }

    /** Headless（v1.1）· 与 Surface Session 分表。 */
    fun createHeadless(): Long {
        if (!isLibraryLoaded()) return 0L
        return runCatching { nativeCreateHeadless() }.getOrDefault(0L)
    }

    fun destroyHeadless(handle: Long) {
        if (handle == 0L || !isLibraryLoaded()) return
        runCatching { nativeDestroyHeadless(handle) }
    }

    fun computeReadback(
        handle: Long,
        wgsl: String,
        input: FloatArray,
        workgroupCountX: Int,
    ): FloatArray? {
        if (handle == 0L || !isLibraryLoaded()) return null
        return runCatching {
            nativeComputeReadback(handle, wgsl, input, workgroupCountX)
        }.getOrNull()
    }

    fun headlessBackendId(handle: Long): String {
        if (handle == 0L || !isLibraryLoaded()) return "wgpu-blocked"
        return runCatching { nativeHeadlessBackendId(handle) }.getOrDefault("wgpu-m0")
    }

    /** ModelPass · 与 Fullscreen Session 分表。 */
    fun createModelFromSurface(surface: Surface, width: Int, height: Int): Long {
        if (!isLibraryLoaded()) return 0L
        return runCatching {
            nativeCreateModel(surface, width.coerceAtLeast(1), height.coerceAtLeast(1))
        }.getOrDefault(0L)
    }

    fun destroyModel(handle: Long) {
        if (handle == 0L || !isLibraryLoaded()) return
        runCatching { nativeDestroyModel(handle) }
    }

    fun resizeModel(handle: Long, width: Int, height: Int) {
        if (handle == 0L) return
        runCatching { nativeResizeModel(handle, width, height) }
    }

    fun setMeshes(handle: Long, verts: FloatArray, indices: IntArray): Boolean {
        if (handle == 0L) return false
        return runCatching { nativeSetMeshes(handle, verts, indices) }.getOrDefault(false)
    }

    fun setFrame(
        handle: Long,
        mvp: FloatArray,
        color: FloatArray,
        lit: Boolean,
        lightDir: FloatArray,
    ) {
        if (handle == 0L) return
        runCatching { nativeSetFrame(handle, mvp, color, lit, lightDir) }
    }

    fun setPlayingModel(handle: Long, playing: Boolean) {
        if (handle == 0L) return
        runCatching { nativeSetPlayingModel(handle, playing) }
    }

    fun tickModel(handle: Long, dtSecs: Float): Boolean {
        if (handle == 0L) return false
        return runCatching { nativeTickModel(handle, dtSecs) }.getOrDefault(false)
    }

    fun backendIdModel(handle: Long): String {
        if (handle == 0L) return "wgpu-blocked"
        return runCatching { nativeBackendIdModel(handle) }.getOrDefault("wgpu-m0")
    }

    fun lastErrorCodeModel(handle: Long): String {
        if (handle == 0L) return ""
        return runCatching { nativeLastErrorCodeModel(handle) }.getOrDefault("")
    }

    fun lastErrorMessageModel(handle: Long): String {
        if (handle == 0L) return ""
        return runCatching { nativeLastErrorMessageModel(handle) }.getOrDefault("")
    }

    fun drainErrorsModel(handle: Long): List<Pair<String, String>> {
        if (handle == 0L) return emptyList()
        val flat = runCatching { nativeDrainErrorsModel(handle) }.getOrNull() ?: return emptyList()
        val out = ArrayList<Pair<String, String>>(flat.size / 2)
        var i = 0
        while (i + 1 < flat.size) {
            out.add(flat[i] to flat[i + 1])
            i += 2
        }
        return out
    }

    private external fun nativeAbiVersion(): Int
    private external fun nativeHasWgpu(): Int
    private external fun nativeCreate(surface: Surface, width: Int, height: Int): Long
    private external fun nativeLastCreateError(): String
    private external fun nativeDestroy(handle: Long)
    private external fun nativeResize(handle: Long, width: Int, height: Int)
    private external fun nativeSetFragmentShader(handle: Long, fs: String): Boolean
    private external fun nativeSetPlaying(handle: Long, playing: Boolean)
    private external fun nativeTick(handle: Long, dtSecs: Float): Boolean
    private external fun nativeBackendId(handle: Long): String
    private external fun nativeLastErrorCode(handle: Long): String
    private external fun nativeLastErrorMessage(handle: Long): String
    private external fun nativeDrainErrors(handle: Long): Array<String>
    private external fun nativeCreateHeadless(): Long
    private external fun nativeDestroyHeadless(handle: Long)
    private external fun nativeHeadlessBackendId(handle: Long): String
    private external fun nativeComputeReadback(
        handle: Long,
        wgsl: String,
        input: FloatArray,
        workgroupCountX: Int,
    ): FloatArray?
    private external fun nativeCreateModel(surface: Surface, width: Int, height: Int): Long
    private external fun nativeDestroyModel(handle: Long)
    private external fun nativeResizeModel(handle: Long, width: Int, height: Int)
    private external fun nativeSetMeshes(handle: Long, verts: FloatArray, indices: IntArray): Boolean
    private external fun nativeSetFrame(
        handle: Long,
        mvp: FloatArray,
        color: FloatArray,
        lit: Boolean,
        lightDir: FloatArray,
    )
    private external fun nativeSetPlayingModel(handle: Long, playing: Boolean)
    private external fun nativeTickModel(handle: Long, dtSecs: Float): Boolean
    private external fun nativeBackendIdModel(handle: Long): String
    private external fun nativeLastErrorCodeModel(handle: Long): String
    private external fun nativeLastErrorMessageModel(handle: Long): String
    private external fun nativeDrainErrorsModel(handle: Long): Array<String>
}

actual object WgpuRuntime {
    actual fun createInstance(): Instance = Instance(AndroidWgpuPlatform)

    actual fun isLibraryLoaded(): Boolean = AndroidWgpuPlatform.isLibraryLoaded()

    actual fun isAvailable(): Boolean = AndroidWgpuPlatform.isAvailable()

    actual fun blockedMessage(): String = AndroidWgpuPlatform.blockedMessage()
}

fun androidPlatform(): AndroidWgpuPlatform = AndroidWgpuPlatform

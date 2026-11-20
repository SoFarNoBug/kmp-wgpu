@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.WgpuRuntime
import com.jlj.kmp.wgpu.ohosPlatform
import kotlinx.cinterop.COpaquePointer

/**
 * @CreateDate: 2026/11/19
 * @Author: jlj
 * @Description: 网格 ModelPass（Ohos OHNativeWindow）；独立于 FullscreenPass。
 */
class ModelPass private constructor(
    private var _handle: Long,
) {
    val handle: Long get() = _handle

    fun resize(width: Int, height: Int) {
        ohosPlatform().resizeModel(_handle, width.coerceAtLeast(1), height.coerceAtLeast(1))
    }

    fun setMeshes(verts: FloatArray, indices: IntArray): Boolean =
        ohosPlatform().setMeshesModel(_handle, verts, indices)

    fun setFrame(mvp: FloatArray, color: FloatArray, lit: Boolean, lightDir: FloatArray) {
        ohosPlatform().setFrameModel(_handle, mvp, color, lit, lightDir)
    }

    fun setPlaying(playing: Boolean) {
        ohosPlatform().setPlayingModel(_handle, playing)
    }

    fun tick(dtSecs: Float): Boolean = ohosPlatform().tickModel(_handle, dtSecs)

    fun backendId(): String = ohosPlatform().backendIdModel(_handle)

    fun lastErrorCode(): String = ohosPlatform().lastErrorCodeModel(_handle)

    fun lastErrorMessage(): String = ohosPlatform().lastErrorMessageModel(_handle)

    fun drainErrors(): List<Pair<String, String>> = ohosPlatform().drainErrorsModel(_handle)

    fun destroy() {
        val h = _handle
        _handle = 0L
        ohosPlatform().destroyModel(h)
    }

    companion object {
        fun create(nativeWindow: COpaquePointer?, width: Int, height: Int): ModelPass? {
            WgpuRuntime.createInstance()
            val handle = ohosPlatform().createModelFromNativeWindow(nativeWindow, width, height)
            if (handle == 0L) return null
            return ModelPass(handle)
        }
    }
}

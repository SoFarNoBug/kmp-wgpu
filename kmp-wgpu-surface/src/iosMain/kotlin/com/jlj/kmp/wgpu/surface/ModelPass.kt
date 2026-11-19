@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.WgpuRuntime
import com.jlj.kmp.wgpu.iosPlatform
import kotlinx.cinterop.COpaquePointer

/**
 * @CreateDate: 2026/11/19
 * @Author: jlj
 * @Description: 网格 ModelPass（iOS Metal）；独立于 FullscreenPass。
 */
class ModelPass private constructor(
    private var _handle: Long,
) {
    val handle: Long get() = _handle

    fun resize(width: Int, height: Int) {
        iosPlatform().resizeModel(_handle, width.coerceAtLeast(1), height.coerceAtLeast(1))
    }

    fun setMeshes(verts: FloatArray, indices: IntArray): Boolean =
        iosPlatform().setMeshesModel(_handle, verts, indices)

    fun setFrame(mvp: FloatArray, color: FloatArray, lit: Boolean, lightDir: FloatArray) {
        iosPlatform().setFrameModel(_handle, mvp, color, lit, lightDir)
    }

    fun setPlaying(playing: Boolean) {
        iosPlatform().setPlayingModel(_handle, playing)
    }

    fun tick(dtSecs: Float): Boolean = iosPlatform().tickModel(_handle, dtSecs)

    fun backendId(): String = iosPlatform().backendIdModel(_handle)

    fun lastErrorCode(): String = iosPlatform().lastErrorCodeModel(_handle)

    fun lastErrorMessage(): String = iosPlatform().lastErrorMessageModel(_handle)

    fun drainErrors(): List<Pair<String, String>> = iosPlatform().drainErrorsModel(_handle)

    fun destroy() {
        val h = _handle
        _handle = 0L
        iosPlatform().destroyModel(h)
    }

    companion object {
        fun create(metalLayer: COpaquePointer?, width: Int, height: Int): ModelPass? {
            WgpuRuntime.createInstance()
            val handle = iosPlatform().createModelFromMetalLayer(metalLayer, width, height)
            if (handle == 0L) return null
            return ModelPass(handle)
        }
    }
}

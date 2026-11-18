package com.jlj.kmp.wgpu.surface

import android.view.Surface
import com.jlj.kmp.wgpu.WgpuRuntime
import com.jlj.kmp.wgpu.androidPlatform

/**
 * @CreateDate: 2026/11/18
 * @Author: jlj
 * @Description: 网格 ModelPass（Android）；独立于 FullscreenPass。
 */
class ModelPass private constructor(
    private var _handle: Long,
) {
    val handle: Long get() = _handle

    fun resize(width: Int, height: Int) {
        androidPlatform().resizeModel(_handle, width.coerceAtLeast(1), height.coerceAtLeast(1))
    }

    /** interleaved pos3+nrm3；indices 为三角索引。 */
    fun setMeshes(verts: FloatArray, indices: IntArray): Boolean =
        androidPlatform().setMeshes(_handle, verts, indices)

    fun setFrame(mvp: FloatArray, color: FloatArray, lit: Boolean, lightDir: FloatArray) {
        androidPlatform().setFrame(_handle, mvp, color, lit, lightDir)
    }

    fun setPlaying(playing: Boolean) {
        androidPlatform().setPlayingModel(_handle, playing)
    }

    fun tick(dtSecs: Float): Boolean = androidPlatform().tickModel(_handle, dtSecs)

    fun backendId(): String = androidPlatform().backendIdModel(_handle)

    fun lastErrorCode(): String = androidPlatform().lastErrorCodeModel(_handle)

    fun lastErrorMessage(): String = androidPlatform().lastErrorMessageModel(_handle)

    fun drainErrors(): List<Pair<String, String>> = androidPlatform().drainErrorsModel(_handle)

    fun destroy() {
        val h = _handle
        _handle = 0L
        androidPlatform().destroyModel(h)
    }

    companion object {
        fun create(surface: Surface, width: Int, height: Int): ModelPass? {
            WgpuRuntime.createInstance()
            val handle = androidPlatform().createModelFromSurface(surface, width, height)
            if (handle == 0L) return null
            return ModelPass(handle)
        }
    }
}

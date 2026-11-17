package com.jlj.kmp.wgpu

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: kmp-wgpu v1 窄子集类型壳（ADR 0013 · 方案 B）。概念近 1:1 wgpu；
 * 热路径为粗粒度能力块，禁止逐 draw FFI。产品面不含 Holm Session 语义。
 */

/** GPU 实例入口（v1：与库加载 / wasm 就绪绑定）。 */
class Instance(internal val platform: WgpuPlatform)

/** 逻辑适配器占位（v1 不单独枚举；随 Device 创建选取）。 */
class Adapter(internal val device: Device)

/**
 * Device：资源创建归属。
 * v1 Android/H5：底层句柄即 holm_fullscreen Session（一次 tick = encode+submit+present）。
 */
class Device(
    val handle: WgpuHandle,
    internal val platform: WgpuPlatform,
) {
    val queue: Queue = Queue(this)

    fun destroy() {
        platform.destroyDevice(handle)
    }

    companion object {
        /** 供 surface 模块 / 适配层包装已有 Session 句柄。 */
        fun wrap(handle: WgpuHandle, platform: WgpuPlatform): Device = Device(handle, platform)
    }
}

/** Queue：提交与 present 归属（v1 经 Device 粗块完成，不单独碎调）。 */
class Queue(internal val device: Device)

/** WGSL ShaderModule 句柄占位（v1 FullscreenPass 内建管线；公开壳供后续扩展）。 */
class ShaderModule(internal val label: String)

/** RenderPipeline 句柄占位。 */
class RenderPipeline(internal val label: String)

/** Buffer 句柄占位（v1 uniforms 仍在 native Session 内）。 */
class Buffer(internal val label: String)

/** Texture 句柄占位。 */
class Texture(internal val label: String)

/** BindGroup 句柄占位。 */
class BindGroup(internal val label: String)

/**
 * CommandEncoder：Kotlin 侧组命令 → 一次 native submit。
 * v1 仅支持全屏 tick 能力块，不暴露逐条 draw FFI。
 */
class CommandEncoder {
    private var _pendingTickDt: Float? = null

    fun recordFullscreenTick(dtSecs: Float) {
        _pendingTickDt = dtSecs
    }

    fun takePendingTickDt(): Float? {
        val v = _pendingTickDt
        _pendingTickDt = null
        return v
    }
}

/** 平台句柄（Android Long / H5 dynamic session）。 */
expect class WgpuHandle

/** 平台能力：库探测与 Device 生命周期粗块。 */
interface WgpuPlatform {
    fun isLibraryLoaded(): Boolean
    fun isAvailable(): Boolean
    fun blockedMessage(): String
    fun destroyDevice(handle: WgpuHandle)
}

expect object WgpuRuntime {
    fun createInstance(): Instance
    fun isLibraryLoaded(): Boolean
    fun isAvailable(): Boolean
    fun blockedMessage(): String
}

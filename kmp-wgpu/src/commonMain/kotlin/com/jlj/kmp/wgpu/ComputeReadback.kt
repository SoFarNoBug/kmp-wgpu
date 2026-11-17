package com.jlj.kmp.wgpu

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: ADR 0013 v1.1 · headless Compute + 回读粗块（方案 B）；与 Present/Holm 正交。
 */

/**
 * 默认示例 WGSL：`out[i] = in[i] * 2`（`@workgroup_size(64)`）。
 * 与 Rust `SAMPLE_DOUBLE_WGSL`（`kmp-wgpu-native/src/compute_readback.rs`）**双份**；改一处须同步另一处。
 */
const val COMPUTE_SAMPLE_DOUBLE_WGSL: String = """
@group(0) @binding(0) var<storage, read> input_buf: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_buf: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&input_buf)) {
        return;
    }
    output_buf[i] = input_buf[i] * 2.0;
}
"""

/** Headless Device 句柄（与 Surface Session 分表）。 */
class HeadlessDevice(
    val handle: WgpuHandle,
    internal val destroyFn: (WgpuHandle) -> Unit,
) {
    // 无 @Volatile：common 兼 JS；同设备粗块串在 Android 单线程池 / H5 Promise 链上。
    private var alive: Boolean = true

    val isAlive: Boolean get() = alive

    /** 幂等：二次及以后 no-op（review P2-destroy-not-idempotent）。 */
    fun destroy() {
        if (!alive) return
        alive = false
        destroyFn(handle)
    }
}

/**
 * 创建 headless Device（无 Surface）。
 * Android：同步（短样本）；H5：无同步路径，请用 [createHeadlessDeviceAsync]。
 */
expect fun createHeadlessDevice(): HeadlessDevice?

/**
 * 异步创建。
 * Android：后台线程执行后回调（[onDone] 可能不在调用线程）；H5：Promise。
 * UI/Kuikly 状态更新须由调用方回线程。
 */
expect fun createHeadlessDeviceAsync(onDone: (Result<HeadlessDevice>) -> Unit)

/**
 * 一次粗块：WGSL compute + f32 回读（同步）。
 * Android：**短同步约定** — 仅短样本；大负载用 [computeReadbackAsync]。
 * 成败只信返回值；勿用 sticky lastError 否决成功。
 */
expect fun HeadlessDevice.computeReadback(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
): Result<FloatArray>

/**
 * 异步粗块。
 * Android：后台线程；H5：Promise。[onDone] 可能不在调用线程。
 */
expect fun HeadlessDevice.computeReadbackAsync(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
    onDone: (Result<FloatArray>) -> Unit,
)

/** 可复现示例：输入 → 期望 ×2；走 Async，Demo 须自行回 UI 线程。 */
object ComputeReadbackSample {
    fun defaultInput(): FloatArray = floatArrayOf(1f, 1f, 1f, 1f, 2f, 3f, 4f, 5f)

    fun workgroupsFor(inputSize: Int): Int = ((inputSize + 63) / 64).coerceAtLeast(1)

    fun run(
        input: FloatArray = defaultInput(),
        onDone: (Result<FloatArray>) -> Unit,
    ) {
        createHeadlessDeviceAsync { created ->
            val device = created.getOrElse {
                onDone(Result.failure(it))
                return@createHeadlessDeviceAsync
            }
            device.computeReadbackAsync(
                shaderWgsl = COMPUTE_SAMPLE_DOUBLE_WGSL,
                input = input,
                workgroupCountX = workgroupsFor(input.size),
            ) { result ->
                device.destroy()
                onDone(result)
            }
        }
    }
}

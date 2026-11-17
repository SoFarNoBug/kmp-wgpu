# kmp-wgpu（ADR 0013 · v1 / v1.1）

与 Holm / Kuikly **无关**的跨端 wgpu **原语库**（Kotlin 脸）。

## 方案 B（已锁定）

- **类型**：近 wgpu 1:1 概念（`Device` / `Queue` / `ShaderModule` / …）
- **桥接**：粗粒度能力块（一次 FullscreenPass.tick = encode + submit + present）；**禁止**逐 `draw` JNI
- **Present**：可选，见 [`../kmp-wgpu-surface`](../kmp-wgpu-surface)

## v1 支持

| API | 状态 |
| --- | --- |
| `WgpuRuntime` / `Instance` / `Device` / `Queue` | 有（Device=Session 句柄） |
| `CommandEncoder.recordFullscreenTick` | 壳；提交走 `FullscreenPass.tick` |
| Buffer / Texture / BindGroup / Pipeline 类型 | 壳占位 |
| 逐条 CommandEncoder FFI | **不做**（方案 C，另 ADR） |

## v1.1 · Headless Compute + 回读

| API | 说明 |
| --- | --- |
| `createHeadlessDevice` / `Async` | 无 Surface 的 Device（与 Present Session **分表**） |
| `HeadlessDevice.computeReadback` / `Async` | 一次粗块：WGSL + `FloatArray` → `FloatArray` |
| `ComputeReadbackSample` | 默认可复现：`out[i]=in[i]*2`（`COMPUTE_SAMPLE_DOUBLE_WGSL`） |

- Android：
  - 同步 API：JNI 内 `pollster::block_on`；**仅短样本**（Demo 默认 8×f32）
  - `*Async`：单线程池 `kmp-wgpu-headless` 执行；`onDone` **可能不在调用线程**（UI/Kuikly 须自行回线程）
  - `HeadlessDevice.destroy`：幂等（二次 no-op）
- H5：`window.__kmpWgpu.createHeadless()` → `HeadlessComputeSession`；只用 Async 路径
- **勿**对 headless 句柄调用 `tick` / `setCustomFs` / Surface resize
- 示例 WGSL：Rust `SAMPLE_DOUBLE_WGSL` 与 Kotlin `COMPUTE_SAMPLE_DOUBLE_WGSL` 双份；改一处须同步另一处

## 非本模块

`setCustomFs` / `setScene` / `liquid*` / `playing` 属 **Holm 适配**，Kotlin 面在 `:holm-renderer-wgpu`（`HolmAndroidJni` / `HolmFullscreenPass`），**不在** `com.jlj.kmp.wgpu`。

## Native

Rust crate 目录为 `kmp-wgpu-native/`，身份 **`kmp_wgpu_native`**；产物 `libkmp_wgpu_native.so`  / `kmp_wgpu_native*.wasm`。

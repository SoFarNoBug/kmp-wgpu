# kmp-wgpu（KMP 原语模块）

跨端 wgpu **原语**（KMP 公共 API）。路人说明见根 [README.md](../README.md)；接入见 [docs/guide.md](../docs/guide.md)；构建发版见 [开发者文档](../CONTRIBUTING.md)。

## 设计要点

- **类型**：贴近 wgpu 概念（`Device` / `Queue` / `ShaderModule` / …）
- **桥接**：粗粒度能力块（一次调用完成 encode + submit；Present 另加 present）
- **Present**：可选，见 [`../kmp-wgpu-surface`](../kmp-wgpu-surface)

## 当前能力

| API | 状态 |
| --- | --- |
| `WgpuRuntime` / `Instance` / `Device` / `Queue` | 有（Device = Session 句柄） |
| `CommandEncoder.recordFullscreenTick` | 壳；真正提交走 `FullscreenPass.tick` |
| Buffer / Texture / BindGroup / Pipeline | 壳占位（规划中补齐） |
| 逐条 CommandEncoder FFI | **非目标**（另议） |

## Headless Compute + 回读

| API | 说明 |
| --- | --- |
| `createHeadlessDevice` / `Async` | 无 Surface 的 Device（与 Present Session **分表**） |
| `HeadlessDevice.computeReadback` / `Async` | 一次粗块：WGSL + `FloatArray` → `FloatArray` |
| `ComputeReadbackSample` | 默认可复现：`out[i]=in[i]*2` |

- Android：同步仅适合**短样本**；大负载用 `*Async`（回调可能不在 UI 线程）
- H5：只用 Async
- **勿**对 headless 句柄调用 `tick` / `setFragmentShader` / Surface resize
- 示例 WGSL 与 Rust `SAMPLE_DOUBLE_WGSL` 双份，改一处须同步

## Native

实现在 `kmp-wgpu-native/`，产物 `libkmp_wgpu_native.so` / `kmp_wgpu_native*.wasm` / xcframework。

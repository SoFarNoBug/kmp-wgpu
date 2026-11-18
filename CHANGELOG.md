# Changelog

## Unreleased

- [ADD] Android `ModelPass`：独立网格 Present（`model_pass.rs` + JNI `nativeCreateModel` / `setMeshes` / `setFrame` / `tickModel`）；Kotlin `com.jlj.kmp.wgpu.surface.ModelPass`；与 FullscreenPass 分表。仅 Android 先通。

## 0.1.0 — 2026-11-17 / 11-18

首发：Headless Compute 回读 + FullscreenPass Present（四端）；故意 ABI break（无 liquid / scene / ClearTri）。

### 分发渠道

| 渠道 | 坐标 / 包 | 状态 |
|---|---|---|
| Maven Central | `io.github.sofarnobug:kmp-wgpu:0.1.0` · `kmp-wgpu-surface:0.1.0` | **已发** |
| git tag | `0.1.0`（含入库 `kmp-wgpu-native/dist/ios/KmpWgpuNative.xcframework`） | **已推** |
| CocoaPods | `KmpWgpuNative` `0.1.0`（仓根 `KmpWgpuNative.podspec`） | **trunk 已收**（CDN / `pod search` 可能滞后数小时） |
| ohpm | `@jlj/kmp-wgpu-native@0.1.1` | **已过审公开**（2026-11-18） |
| npm | `@sofarnobug/kmp-wgpu-native@0.1.0` | **已发**（scope=`sofarnobug`；与 ohpm `@jlj` 不同） |
| GitHub Release | [0.1.0](https://github.com/SoFarNoBug/kmp-wgpu/releases/tag/0.1.0) | **已建**（2026-11-18） |

### 其它

- iOS Pod：最低部署目标 **15.0**；xcframework 仅 **arm64**（含 sim），podspec 排除模拟器 `x86_64`
- 鸿蒙 ohpm `0.1.1`：相对 `0.1.0` 补 `dependencies: {}`（审核必填）；**已过审**，`ohpm install` 可走公开源

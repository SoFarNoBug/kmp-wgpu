# kmp-wgpu · 接入指南

给**已决定接入**的工程同学。产品定位与最小示例见根 [README.md](../README.md)。构建 / 发版 → [开发者文档](../CONTRIBUTING.md)。

## 前置

1. **Kotlin Multiplatform**（或单端 Kotlin）宿主。
2. 按端打入 **native**（`.so` / xcframework / wasm）。只加 Maven、不带 native → create 失败或 `UnsatisfiedLinkError`。

## 各端依赖清单

| 端 | Maven | Native | 接完标准 |
|---|---|---|---|
| **Android** | `kmp-wgpu`（+ Present 时 `kmp-wgpu-surface`） | `jniLibs/**/libkmp_wgpu_native.so`（当前臂：arm64） | Headless 可回读；或 `AndroidSurfaces.createFullscreenPass` 出画，`backendId()` 可读 |
| **iOS** | 同上（进 KN / KMP framework） | CocoaPods `KmpWgpuNative` **0.1.0** | `IosSurfaces` + Metal；开 **xcworkspace** |
| **鸿蒙** | 同上（ohosArm64） | ohpm `@jlj/kmp-wgpu-native@0.1.1` | XComponent → `OHNativeWindow` → `OhosSurfaces`；外泵 OnFrame |
| **H5** | 同上（JS） | npm `@jlj/kmp-wgpu-native@0.1.0`（或静态拷贝 js/wasm） | `JsSurfaces.create`；Headless **仅 Async** |

```kotlin
repositories { mavenCentral() }

dependencies {
    implementation("io.github.sofarnobug:kmp-wgpu:0.1.0")
    implementation("io.github.sofarnobug:kmp-wgpu-surface:0.1.0")
}
```

### iOS

```ruby
pod 'KmpWgpuNative', '0.1.0'
```

必须用 `*.xcworkspace`。Kotlin framework 需能链到 vendored xcframework。

### 鸿蒙

```bash
ohpm install @jlj/kmp-wgpu-native@0.1.1
```

渲染目标经 **XComponent** 取 `OHNativeWindow`。当前稳妥路径是 **GLES + EGL**（Mate 60 Pro 实测 GLES3）；Vulkan 需专门适配，见根 README 后端表与本仓 `ohos_c_api.rs` 注释。

### H5

```bash
npm i @jlj/kmp-wgpu-native@0.1.0
```

将 `kmp_wgpu_native.js` + `kmp_wgpu_native_bg.wasm` 放到可加载路径，并挂好 `window.__kmpWgpu` 胶水（对照 Demo / H5 壳）。

## 概念：两条路径

```text
Headless（无窗口）
  createHeadlessDevice[Async]
    → computeReadback[Async](WGSL, FloatArray)
    → Result<FloatArray> → destroy()

Present（有窗口）
  *Surfaces.create(...) → GpuSurface
    → FullscreenPass
    → setFragmentShader(WGSL)
    → 每帧 tick(dt)   ← 宿主外泵
```

**粗粒度约定：** 一次 `tick` / 一次 `computeReadback` = 一次完整 GPU 回合。不要期望在 Kotlin 里拼半条 render pass。

Present Session 与 Headless Device **分表**：勿对 headless 句柄调用 `tick` / `setFragmentShader` / Surface resize。

## Headless Compute

```kotlin
import com.jlj.kmp.wgpu.COMPUTE_SAMPLE_DOUBLE_WGSL
import com.jlj.kmp.wgpu.ComputeReadbackSample
import com.jlj.kmp.wgpu.createHeadlessDeviceAsync
import com.jlj.kmp.wgpu.computeReadbackAsync

ComputeReadbackSample.run { /* … */ }

createHeadlessDeviceAsync { created ->
    val device = created.getOrElse { return@createHeadlessDeviceAsync }
    device.computeReadbackAsync(
        shaderWgsl = COMPUTE_SAMPLE_DOUBLE_WGSL,
        input = floatArrayOf(1f, 2f, 3f, 4f),
        workgroupCountX = 1,
    ) { result ->
        device.destroy() // 幂等
        // onDone 可能不在 UI 线程
    }
}
```

| 端 | 同步 API | 异步 |
|---|---|---|
| Android | 有（**仅短样本**） | 线程池 `kmp-wgpu-headless` |
| H5 | **无** | Promise / `*Async` |
| iOS / 鸿蒙 | 随 Kotlin 脸 | 同左 |

示例 WGSL `COMPUTE_SAMPLE_DOUBLE_WGSL` 与 Rust `SAMPLE_DOUBLE_WGSL` **双份**；改一处须同步另一处。

## FullscreenPass

| 端 | 工厂 |
|---|---|
| Android | `AndroidSurfaces.create` / `createFullscreenPass(surface, w, h)` |
| iOS | `IosSurfaces.create(metalLayer, w, h)` |
| 鸿蒙 | `OhosSurfaces.create(nativeWindow, w, h)` |
| H5 | `JsSurfaces.create(canvas, w, h, onSuccess, onError)` |

```kotlin
val pass = AndroidSurfaces.createFullscreenPass(surface, w, h)
    ?: error(AndroidSurfaces.lastCreateError())

if (!pass.setFragmentShader(wgslFs)) {
    // lastErrorCode / lastErrorMessage / drainErrors
}
pass.tick(dtSecs)
pass.backendId() // Vulkan / GL / Metal / BrowserWebGpu …
pass.destroy()
```

- 无片元时 `tick` **只清屏**（合法空状态）。
- 帧泵由宿主驱动（Choreographer / CADisplayLink / XComponent OnFrame / `requestAnimationFrame`）。
- 改 Rust 后须重跑对应 `kmp-wgpu-native/build-*.sh`；Gradle **不**自动编 Rust。

## 后端怎么查

永远信任运行时：

```kotlin
pass.backendId()
// headless 侧亦有 backend id API（见平台绑定）
```

不要假设 Android=Vulkan、鸿蒙=Vulkan。说明见根 README「各端 · 实际渲染后端」。

## 接入避坑

| 坑 | 现象 | 怎么避 |
|---|---|---|
| 只加 Maven、忘 native | create 失败 / UnsatisfiedLinkError | 按上表补 `.so` / Pod / ohpm / wasm |
| H5 用同步 Headless | 不可用 | 一律 `*Async` |
| Android 主线程大 compute | 卡顿 / ANR 风险 | 短样本同步；大负载 Async |
| 回调里直接改 UI | 线程错误 | `onDone` 可能不在调用线程 |
| headless 去 `tick` | 未定义 | 分表，各用各的 API |
| 期望细粒度 Buffer/draw FFI | 类型多为壳 | 0.1.0 只承诺粗块 |
| iOS 开裸 `xcodeproj` | 链不上 | 开 **xcworkspace** |
| 改了 Rust 只编 Kotlin | 旧 `.so` 行为 | 重跑 `build-*.sh` |
| 鸿蒙当 Vulkan 必成 | 建 surface 失败 | 当前主路径 GLES；查 `backendId()` |

## 仓库指路

| 路径 | 角色 |
|---|---|
| [`sample/`](../sample/) | 独立冒烟说明 |
| [`kmp-wgpu/`](../kmp-wgpu/) | Kotlin 原语（含 Headless） |
| [`kmp-wgpu-surface/`](../kmp-wgpu-surface/) | Surface + FullscreenPass |
| [`kmp-wgpu-native/`](../kmp-wgpu-native/) | Rust · 各端产物 |

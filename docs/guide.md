# kmp-wgpu · 接入指南

给**已决定接入**的工程同学。产品定位与最小示例见根 [README.md](../README.md)。构建 / 发版 → [开发者文档](../CONTRIBUTING.md)。

## 前置

1. **Kotlin Multiplatform**（或单端 Kotlin）宿主。
2. 按端打入 **native**（`.so` / xcframework / wasm）。只加 Maven、不带 native → create 失败或 `UnsatisfiedLinkError`。
3. 0.1.0 只承诺两条**粗粒度**能力块（见下）；`Buffer` / `Texture` / `Pipeline` 等类型多为壳，不要按完整 WebGPU 细 API 来用。

## 各端依赖清单

| 端 | Maven | Native | 接完标准 |
|---|---|---|---|
| **Android** | `kmp-wgpu`（+ Present 时 `kmp-wgpu-surface`） | `jniLibs/**/libkmp_wgpu_native.so`（当前臂：arm64） | Headless 可回读；或 `AndroidSurfaces.createFullscreenPass` 出画，`backendId()` 可读 |
| **iOS** | 同上（进 KN / KMP framework） | CocoaPods `KmpWgpuNative` **0.1.0**（trunk 已收；CDN 可能滞后） | `IosSurfaces` + Metal；开 **xcworkspace** |
| **鸿蒙** | 同上（ohosArm64） | ohpm `@jlj/kmp-wgpu-native@0.1.1`（**审核中**） | XComponent → `OHNativeWindow` → `OhosSurfaces`；外泵 OnFrame |
| **H5** | 同上（JS） | npm `@jlj/kmp-wgpu-native@0.1.0`（**待发**；可拷贝 js/wasm） | `JsSurfaces.create`；Headless **仅 Async** |

```kotlin
repositories { mavenCentral() }

dependencies {
    implementation("io.github.sofarnobug:kmp-wgpu:0.1.0")
    // Present 时再加：
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

### 包体积（native 大头）

体积主要来自 **`kmp-wgpu-native`**，不是 Kotlin 薄脸。本仓现有产物实测（2026-11-18；未压缩单文件）：

| 端 / 产物 | 约 |
|---|---|
| Android / 鸿蒙 arm64 `libkmp_wgpu_native.so` | **~4.0～4.05 MB** |
| H5 `kmp_wgpu_native_bg.wasm` | **~2.32 MB** |
| iOS xcframework | 以实秤为准，同属数 MB 级 |

只加 Maven、不打 native → 包体几乎不增，但运行时不可用。多 ABI 近似按份数叠加。

## 概念：两条路径（分表）

```text
Headless（无窗口）—— 包 kmp-wgpu
  createHeadlessDevice[Async]
    → computeReadback[Async](WGSL, FloatArray)
    → Result<FloatArray> → destroy()

Present（有窗口）—— 再加 kmp-wgpu-surface
  *Surfaces.create(...) → GpuSurface
    → FullscreenPass
    → setFragmentShader(WGSL)
    → 每帧 tick(dt)   ← 宿主外泵
```

**粗粒度约定：** 一次 `tick` / 一次 `computeReadback` = 一次完整 GPU 回合（encode + submit；Present 再加 present）。不要期望在 Kotlin 里拼半条 render pass。

**分表：** Present Session 与 Headless Device **不是同一类句柄**。勿对 headless 调用 `tick` / `setFragmentShader` / Surface `resize`；也不要对 Present Device 走 `computeReadback*`。

包名：

| 模块 | 包 |
|---|---|
| `kmp-wgpu` | `com.jlj.kmp.wgpu` |
| `kmp-wgpu-surface` | `com.jlj.kmp.wgpu.surface` |

---

## API · 运行时探测：`WgpuRuntime`

```kotlin
expect object WgpuRuntime {
    fun createInstance(): Instance
    fun isLibraryLoaded(): Boolean
    fun isAvailable(): Boolean
    fun blockedMessage(): String
}
```

| 方法 | 含义 |
|---|---|
| `isLibraryLoaded()` | native / wasm 胶水是否已加载 |
| `isAvailable()` | 库已加载且底层认为可用（否则勿硬 create） |
| `blockedMessage()` | 不可用时的人话原因（缺 `.so`、`has_wgpu=0` 等） |
| `createInstance()` | 拿到 `Instance`（v1 多与库就绪绑定；Present 工厂内部也会调） |

失败应**可见**，不要静默换成别的图形栈。

---

## API · Headless Compute（`kmp-wgpu`）

### `HeadlessDevice`

| 成员 | 含义 |
|---|---|
| `handle` | 平台句柄（与 Present Session **分表**） |
| `isAlive` | 是否尚未 `destroy` |
| `destroy()` | **幂等**：二次及以后 no-op |

### 创建

| API | 同步/异步 | 说明 |
|---|---|---|
| `createHeadlessDevice(): HeadlessDevice?` | 同步 | Android：短样本可用；**H5 无同步路径** → 请用 Async |
| `createHeadlessDeviceAsync(onDone: (Result<HeadlessDevice>) -> Unit)` | 异步 | Android：后台线程后回调；H5：Promise。**`onDone` 可能不在 UI 线程** |

### 粗块回读

```kotlin
fun HeadlessDevice.computeReadback(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
): Result<FloatArray>

fun HeadlessDevice.computeReadbackAsync(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
    onDone: (Result<FloatArray>) -> Unit,
)
```

| 参数 | 含义 |
|---|---|
| `shaderWgsl` | WGSL **compute** 着色器全文 |
| `input` | 输入 `f32` 数组（进 storage buffer） |
| `workgroupCountX` | X 维 workgroup 数量（须覆盖 `input.size`；见样本助手） |
| 返回 / `onDone` | `Result<FloatArray>`：成功为输出数组；失败看 `exception`。**成败只信返回值**，勿用 sticky lastError 否决成功 |

| 端 | 同步 `computeReadback` | 异步 |
|---|---|---|
| Android | 有（**仅短样本**；大负载易卡主线程） | 线程池 `kmp-wgpu-headless` |
| H5 | **无** | 必须 `*Async` |
| iOS / 鸿蒙 | 随绑定提供 | 可用 |

### 示例与常量

| 符号 | 含义 |
|---|---|
| `COMPUTE_SAMPLE_DOUBLE_WGSL` | 默认可复现：`out[i] = in[i] * 2`（`@workgroup_size(64)`）。与 Rust `SAMPLE_DOUBLE_WGSL` **双份**，改一处须同步 |
| `ComputeReadbackSample.defaultInput()` | 默认输入数组 |
| `ComputeReadbackSample.workgroupsFor(inputSize)` | 按 64 对齐算 `workgroupCountX` |
| `ComputeReadbackSample.run(input, onDone)` | 一键：Async 创建 → 跑样本 → `destroy` → 回调（Demo 须自行回 UI 线程） |

```kotlin
import com.jlj.kmp.wgpu.COMPUTE_SAMPLE_DOUBLE_WGSL
import com.jlj.kmp.wgpu.ComputeReadbackSample
import com.jlj.kmp.wgpu.createHeadlessDeviceAsync
import com.jlj.kmp.wgpu.computeReadbackAsync

ComputeReadbackSample.run { result ->
    result.onSuccess { out -> /* 使用 out */ }
        .onFailure { err -> /* 处理失败 */ }
}

createHeadlessDeviceAsync { created ->
    val device = created.getOrElse { return@createHeadlessDeviceAsync }
    device.computeReadbackAsync(
        shaderWgsl = COMPUTE_SAMPLE_DOUBLE_WGSL,
        input = floatArrayOf(1f, 2f, 3f, 4f),
        workgroupCountX = 1,
    ) { result ->
        device.destroy()
        // onDone 可能不在 UI 线程
    }
}
```

---

## API · Present：工厂与表面（`kmp-wgpu-surface`）

### 各端工厂

| 端 | 对象 | 创建 | 错误 |
|---|---|---|---|
| Android | `AndroidSurfaces` | `create(surface, w, h): GpuSurface?` · **`createFullscreenPass(surface, w, h): FullscreenPass?`** | `lastCreateError(): String` |
| iOS | `IosSurfaces` | `create(metalLayer, w, h): GpuSurface?`（`COpaquePointer?` → `CAMetalLayer*`） | `lastCreateError()` |
| 鸿蒙 | `OhosSurfaces` | `create(nativeWindow, w, h): GpuSurface?`（`OHNativeWindow*`） | `lastCreateError()` |
| H5 | `JsSurfaces` | **`create(canvas, w, h, onSuccess, onError)`**（异步） | 失败走 `onError(message)` |

宽高内部会 `coerceAtLeast(1)`。create 返回 `null` / 走 `onError` 时先读 `lastCreateError()` 或错误回调，再查 `WgpuRuntime.blockedMessage()`。

iOS / 鸿蒙没有 `createFullscreenPass` 快捷方法时，用：

```kotlin
val gpu = IosSurfaces.create(metalLayer, w, h)
    ?: error(IosSurfaces.lastCreateError())
val pass = FullscreenPass(gpu)
```

### `SurfaceConfiguration`

```kotlin
data class SurfaceConfiguration(
    val width: Int,
    val height: Int,
)
```

v1 仅宽高（近 wgpu SurfaceConfiguration 子集）。

### `GpuSurface`

| 成员 | 含义 |
|---|---|
| `device` | 绑定的 `Device`（与表面同生同灭） |
| `configuration` | 当前宽高配置 |
| `configure(config)` | 更新宽高并 `resize` 到底层；宽高至少为 1 |

v1：Device 在 create 时已绑窗口 / canvas；销毁走 `FullscreenPass.destroy()` / `device.destroy()`。

---

## API · `FullscreenPass`

业务 Present 的主入口（包 `com.jlj.kmp.wgpu.surface`）。

```kotlin
class FullscreenPass(val gpuSurface: GpuSurface) {
    val device: Device
    fun resize(width: Int, height: Int)
    fun setFragmentShader(wgslFs: String): Boolean
    fun setPlaying(playing: Boolean)
    fun tick(dtSecs: Float): Boolean
    fun backendId(): String
    fun lastErrorCode(): String
    fun lastErrorMessage(): String
    fun drainErrors(): List<Pair<String, String>>
    fun destroy()
}
```

| 方法 | 含义 |
|---|---|
| `resize(w, h)` | 改绘制缓冲尺寸（委托 `gpuSurface.configure`） |
| `setFragmentShader(wgslFs)` | 热换 **WGSL 片元**。成功 `true`；失败 `false`，再读 `lastError*` / `drainErrors` |
| `setPlaying(playing)` | 暂停 / 恢复底层播放语义（宿主仍可停自己的外泵） |
| `tick(dtSecs)` | **一轮**完整 Present（encode + submit + present）。`dtSecs` 为帧间隔秒。成功 `true` |
| `backendId()` | 运行时真实后端字符串（如 Vulkan / GL / Metal / BrowserWebGpu 一类）；**以返回值为准**，勿写死端=某 API |
| `lastErrorCode()` / `lastErrorMessage()` | 最近一次错误码 / 文案 |
| `drainErrors()` | 取出并清空积压错误列表 `(code, message)` |
| `destroy()` | 销毁底层 Device（释放 Session） |

约定：

- **尚未** `setFragmentShader` 时，`tick` **只清屏**（合法空状态）。
- 帧泵由**宿主**驱动：Choreographer / CADisplayLink / XComponent OnFrame / `requestAnimationFrame`。
- 改 Rust 后须重跑 `kmp-wgpu-native/build-*.sh`；Gradle **不**自动编 Rust。

```kotlin
import com.jlj.kmp.wgpu.surface.AndroidSurfaces

val pass = AndroidSurfaces.createFullscreenPass(surface, w, h)
    ?: error(AndroidSurfaces.lastCreateError())

if (!pass.setFragmentShader(wgslFs)) {
    // pass.lastErrorCode() / lastErrorMessage() / drainErrors()
}
pass.tick(dtSecs)
println(pass.backendId())
pass.destroy()
```

片元最小形态示例：

```wgsl
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
    return vec4f(0.2, 0.55, 0.9, 1.0)
}
```

（顶点侧由库固定全屏 VS；具体入参名以当前 native 管线约定为准，对照 Demo / 样本。）

---

## API · 类型壳（0.1.0 多数不可当细 FFI 用）

包 `com.jlj.kmp.wgpu` 里还有近 wgpu 的概念壳，**热路径仍是上面两条粗块**：

| 类型 | 0.1.0 状态 |
|---|---|
| `Instance` / `Adapter` / `Device` / `Queue` | Device 有 `handle`、`queue`、`destroy()`；Present 经 surface 工厂创建 |
| `ShaderModule` / `RenderPipeline` / `Buffer` / `Texture` / `BindGroup` | **占位壳**（多仅 `label`） |
| `CommandEncoder` | 仅内部式 `recordFullscreenTick` / `takePendingTickDt`；**无**逐条 draw FFI |
| `WgpuHandle` | expect：Android `Long`；H5 为 dynamic session 等 |

需要细粒度 CommandEncoder / 多物体渲染时：视为**规划中**，不要在 0.1.0 上硬拧。

---

## 后端怎么查

永远信任运行时：

```kotlin
pass.backendId()
WgpuRuntime.isAvailable()
WgpuRuntime.blockedMessage()
```

不要假设 Android=Vulkan、鸿蒙=Vulkan。说明见根 README「各端 · 实际渲染后端」。

---

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
| 只要 Kuikly 页内岛 | 接错层 | 用 [Holm](https://github.com/SoFarNoBug/KuiklyHolm)（其 wgpu 轨再依赖本库） |

## 仓库指路

| 路径 | 角色 |
|---|---|
| [`sample/`](../sample/) | 独立冒烟说明 |
| [`kmp-wgpu/`](../kmp-wgpu/) | Kotlin 原语（含 Headless） |
| [`kmp-wgpu-surface/`](../kmp-wgpu-surface/) | Surface + FullscreenPass |
| [`kmp-wgpu-native/`](../kmp-wgpu-native/) | Rust · 各端产物 |
| 根 [README.md](../README.md) | 路人 |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | 维护 / 发版 |

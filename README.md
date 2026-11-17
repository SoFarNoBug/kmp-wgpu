# kmp-wgpu

**一份 WGSL，四端 GPU。**

**KMP 跨平台**的现代 GPU 入口：业务侧写同一套 Kotlin API + [WGSL](https://www.w3.org/TR/WGSL/)，底层是 [wgpu](https://wgpu.rs/)——兼容 [WebGPU](https://www.w3.org/TR/webgpu/) 的跨端实现。原生可落到 **Vulkan / Metal / D3D12 / OpenGL ES**，浏览器可落到 **WebGPU / WebGL2**。

你可以用它：

- **算**：无窗口把大批并行计算丢给 GPU，结果再拿回 CPU（Headless Compute）
- **画**：往 Surface / MetalLayer / XComponent / Canvas 上跑全屏片元，热换 shader（FullscreenPass）

任意 KMP 工程或原生壳都能接；特效内容、业务算法由你自己写 WGSL，库不内置特效目录。

| | |
|---|---|
| Maven | `io.github.sofarnobug:kmp-wgpu:0.1.0` · `kmp-wgpu-surface:0.1.0` |
| Native | Android `.so` · CocoaPods `KmpWgpuNative` · ohpm / npm `@jlj/kmp-wgpu-native` |
| 语言 | Kotlin Multiplatform 脸 + Rust/`wgpu` 实现 |
| License | MIT |

> **路人读本页。** 完整接入 → [docs/guide.md](docs/guide.md)。构建 / 发版 → [开发者文档](CONTRIBUTING.md)。

---

## 为什么是 wgpu / WebGPU？

传统移动端图形栈往往「每端一套」：Android 啃 Vulkan 或 GLES，iOS 啃 Metal，H5 啃 WebGL，鸿蒙再对接自己的 NativeWindow。算法与特效很难共用。

[WebGPU](https://gpuweb.github.io/gpuweb/explainer/) 把现代 GPU 能力收成**统一的概念模型**（Device / Queue / Buffer / Texture / Compute & Render Pipeline），并用 **WGSL** 写着色器。[wgpu](https://wgpu.rs/) 是这套模型在 Rust 侧的成熟实现：同一份逻辑可在桌面、移动、浏览器上跑。

相对老一代「几乎只为画图」的 API，WebGPU / wgpu 还把 **通用计算（GPGPU）** 做成一等公民——不只后处理滤镜，也包括并行数值、仿真步进、端侧加速等方向（生态里已有图像处理、流体、浏览器侧 ML 推理等实践）。

**kmp-wgpu** 做的事：把这条路接到 **Kotlin Multiplatform**，并先交付两条最常用的**粗粒度能力块**（一次调用 = 完整 GPU 回合），而不是一上来暴露逐条 `draw` 的细 FFI。

```text
你的 KMP / App
    │  Kotlin API（com.jlj.kmp.wgpu*）
    ▼
kmp-wgpu / kmp-wgpu-surface
    │  JNI / C / wasm-bindgen
    ▼
kmp_wgpu_native（Rust + wgpu 30）
    │
    ├─ Android ──► Vulkan 或 GLES（视驱动 / 机型）
    ├─ iOS ──────► Metal
    ├─ 鸿蒙 ─────► 当前主路径 GLES（EGL + XComponent）
    └─ H5 ───────► WebGPU，否则 WebGL2
```

---

## wgpu 能干什么？本库交到哪一步？

下表按**用户能感到的结果**写场景，并标明本库状态（**已有** / **规划中** / **非目标**）。规划中的能力算进产品愿景，但不等于 0.1.0 开箱即用。

| 场景 | 状态 | 说明 |
|---|---|---|
| 全屏 / 局部自定义片元特效、后处理、可视化 | **已有** | `FullscreenPass`：换 WGSL 片元 + 宿主外泵 `tick` |
| 无窗口并行数值计算（结果回 CPU） | **已有** | Headless：`FloatArray` → GPU → `FloatArray` |
| 图像 / 视频类并行处理、LUT、卷积尖刺 | **规划中** | 今日可用粗块 compute 打样；更丰富缓冲 / 纹理路径待补 |
| 粒子 / 物理 / 流体等仿真（compute + 可选 Present） | **规划中** | 能力在 WebGPU 模型内；本库细粒度 dispatch 未齐 |
| 端侧 ML 推理加速（GPGPU） | **规划中** | 生态常见于 WebGPU compute；本库尚未做成推理 SDK |
| 多物体 CAD / 游戏级完整渲染引擎 | **规划中** | 类型壳已占位；逐条 draw FFI 另议 |
| 系统级毛玻璃 / 内置业务特效目录 | **非目标** | 特效内容由业务自己写 WGSL |

**0.1.0 已交付两条粗块：**

1. **Headless Compute 回读**——无窗口算完拿数  
2. **FullscreenPass Present**——有窗口换片元出画  

`Buffer` / `Texture` / `Pipeline` 等类型多为壳；更细的 CommandEncoder FFI 不在本波范围。细节见模块 [kmp-wgpu/README.md](kmp-wgpu/README.md)。

---

## 各端 · 实际渲染后端

应用始终面对 **wgpu API**；真正推像素的是平台后端。**以运行时 `backendId()` 为准**，文档不写死「某端 = 某 API」。

| 端 | 本仓常见情况 | 备注 |
|---|---|---|
| **Android** | **Vulkan 或 OpenGL ES** | 开启 `VULKAN \| GL`；真机常优先 Vulkan，驱动差 / 模拟器等可能落到 GLES |
| **iOS** | **Metal** | Present 绑 `CAMetalLayer` |
| **鸿蒙** | **OpenGL ES（EGL + XComponent）** | Mate 60 Pro **实测 GLES3**。鸿蒙原生也支持 Vulkan，但 wgpu 侧需专门 surface 适配；当前路径上 Vulkan 建 surface 常失败 → 回退 **GLES**（社区稳妥方案） |
| **H5** | **WebGPU**，否则 **WebGL2** | 视浏览器能力（日志里可见类似 `wgpu-m0` / `wgpu-m0-gl`） |

查真相：

```kotlin
pass.backendId()
```

---

## 模块怎么选

```text
只要算完拿数（无窗口）  →  kmp-wgpu + 该端 native
还要画到屏上            →  再加 kmp-wgpu-surface
```

| 模块 | Maven / 分发 | 你得到什么 |
|---|---|---|
| **kmp-wgpu** | `io.github.sofarnobug:kmp-wgpu:0.1.0` | Headless Device、`computeReadback*`、Runtime / Device 句柄 |
| **kmp-wgpu-surface** | `io.github.sofarnobug:kmp-wgpu-surface:0.1.0` | `*Surfaces`、`FullscreenPass` |
| **kmp-wgpu-native** | 按端：`.so` / Pod / ohpm / npm | Rust 运行时（`libkmp_wgpu_native`） |

```kotlin
repositories { mavenCentral() }

dependencies {
    implementation("io.github.sofarnobug:kmp-wgpu:0.1.0")
    // 要 Present 时再加：
    implementation("io.github.sofarnobug:kmp-wgpu-surface:0.1.0")
}
```

### Native 一眼表

| 端 | 怎么加 native |
|---|---|
| Android | 打包 `jniLibs/**/libkmp_wgpu_native.so`（当前主推 arm64） |
| iOS | `pod 'KmpWgpuNative', '0.1.0'`（请用 **xcworkspace**） |
| 鸿蒙 | `ohpm install @jlj/kmp-wgpu-native@0.1.1` |
| H5 | `npm i @jlj/kmp-wgpu-native@0.1.0`（或拷贝 js + wasm） |

完整步骤与避坑 → [docs/guide.md](docs/guide.md)。

---

## 两条路径（概念）

```text
┌─ Headless ─────────────────────────────────────┐
│  createHeadlessDevice[Async]                   │
│       ↓                                        │
│  computeReadback[Async](WGSL, FloatArray)      │
│       ↓                                        │
│  Result<FloatArray>  →  destroy()              │
└────────────────────────────────────────────────┘

┌─ Present ──────────────────────────────────────┐
│  *Surfaces.create(窗口)  →  GpuSurface          │
│       ↓                                        │
│  FullscreenPass                                │
│       ↓ setFragmentShader(WGSL)                │
│  每帧 tick(dt)  ← 宿主外泵                     │
│     （Choreographer / DisplayLink / OnFrame）  │
└────────────────────────────────────────────────┘
```

约定：

- **一次** `tick` / **一次** `computeReadback` = **一轮**完整 GPU 工作（encode + submit，Present 再加 present）
- Headless 与 Present **分表**：不要对 headless 句柄调用 `tick` / 换片元 / resize

---

## 最小接入 · Headless Compute

把 `FloatArray` 送进 GPU，按 WGSL 算完再拿回来。默认可复现示例：`out[i] = in[i] * 2`。

```kotlin
import com.jlj.kmp.wgpu.ComputeReadbackSample

ComputeReadbackSample.run { result ->
    result
        .onSuccess { out -> /* 使用 out */ }
        .onFailure { err -> /* 处理失败 */ }
}
```

需要自己控生命周期时：

```kotlin
import com.jlj.kmp.wgpu.COMPUTE_SAMPLE_DOUBLE_WGSL
import com.jlj.kmp.wgpu.createHeadlessDeviceAsync
import com.jlj.kmp.wgpu.computeReadbackAsync

createHeadlessDeviceAsync { created ->
    val device = created.getOrElse { return@createHeadlessDeviceAsync }
    device.computeReadbackAsync(
        shaderWgsl = COMPUTE_SAMPLE_DOUBLE_WGSL,
        input = floatArrayOf(1f, 2f, 3f, 4f),
        workgroupCountX = 1,
    ) { result ->
        device.destroy() // 幂等
        // 注意：回调可能不在 UI 线程
    }
}
```

| 端 | 同步 API | 异步 |
|---|---|---|
| Android | 有（仅短样本） | 推荐大负载走 Async |
| H5 | 无 | 必须用 `*Async` |
| iOS / 鸿蒙 | 有（随绑定） | 可用 |

---

## 最小接入 · FullscreenPass（Android）

拿到 `Surface` 后创建 Pass、换片元、由宿主泵帧：

```kotlin
import com.jlj.kmp.wgpu.surface.AndroidSurfaces

val pass = AndroidSurfaces.createFullscreenPass(surface, width, height)
    ?: error(AndroidSurfaces.lastCreateError())

val ok = pass.setFragmentShader(
    """
    @fragment
    fn fs_main(in: VsOut) -> @location(0) vec4f {
        return vec4f(0.2, 0.55, 0.9, 1.0)
    }
    """.trimIndent(),
)
if (!ok) {
    // pass.lastErrorCode() / lastErrorMessage()
}

// 每帧（Choreographer 等）
pass.tick(dtSecs)
println(pass.backendId())
```

其它端工厂同构：

| 端 | 工厂 |
|---|---|
| Android | `AndroidSurfaces.create` / `createFullscreenPass` |
| iOS | `IosSurfaces.create(metalLayer, w, h)` |
| 鸿蒙 | `OhosSurfaces.create(nativeWindow, w, h)` |
| H5 | `JsSurfaces.create(canvas, w, h, onSuccess, onError)` |

- 尚未设置片元时，`tick` **只清屏**（合法空状态）
- `resize` / `setPlaying` / `destroy` 见 `FullscreenPass` API
- 改底层 Rust 后须重跑 `kmp-wgpu-native/build-*.sh`（Gradle 不编 Rust）

---

## 能力矩阵

| | Android | iOS | 鸿蒙 | H5 |
|---|---|---|---|---|
| Headless Compute | ✅ | ✅ | ✅ | ✅（仅 Async） |
| FullscreenPass | ✅ Surface | ✅ CAMetalLayer | ✅ OHNativeWindow | ✅ Canvas |
| 片元热换 | ✅ | ✅ | ✅ | ✅ |
| 运行时查后端 | `backendId()` | 同左 | 同左 | 同左 |
| 缺 native | 失败可见 | 失败可见 | 失败可见 | 失败可见 |

---

## 常见问题（路人）

**Q. 这和直接用 Vulkan / Metal / WebGL 有什么差别？**  
A. 你写的是 **统一的 wgpu / WGSL**；各端后端由 wgpu 选择。业务算法与特效能跨端复用，而不是四套原生 API 各写一遍。

**Q. 0.1.0 能做完整游戏引擎吗？**  
A. 还不能按「细粒度 draw 调用」来用。当前适合：**全屏片元特效**、**无窗口 compute 回读**，以及在此之上的尖刺 / Demo。更细的 Buffer / Pipeline FFI 在规划中。

**Q. 鸿蒙是不是 Vulkan？**  
A. **以真机 `backendId()` 为准。** Mate 60 Pro 实测为 **OpenGL ES 3**；Vulkan 在鸿蒙理论可行，但需专门适配，本仓当前主路径是 GLES。

**Q. Android 是不是一定 Vulkan？**  
A. **不一定。** 本仓同时启用 Vulkan 与 GLES，由适配器择优。

**Q. 详细接入、避坑去哪看？**  
A. → [docs/guide.md](docs/guide.md)。编 Rust / 发版 → [开发者文档](CONTRIBUTING.md)。

---

## 仓库地图

| 路径 | 角色 |
|---|---|
| [`kmp-wgpu/`](kmp-wgpu/) | KMP 原语（含 Headless） |
| [`kmp-wgpu-surface/`](kmp-wgpu-surface/) | Surface + FullscreenPass |
| [`kmp-wgpu-native/`](kmp-wgpu-native/) | Rust 实现与各端产物 |
| [`sample/`](sample/) | 独立冒烟说明 |
| [`docs/guide.md`](docs/guide.md) | 接入手册 |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | 开发者文档 |

---

接入细节与避坑 → **[docs/guide.md](docs/guide.md)**  
构建 / 发版 / 共建 → **[开发者文档](CONTRIBUTING.md)**

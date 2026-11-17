# kmp-wgpu

跨端 wgpu 原语库（Kotlin 脸）+ 可选 Present。**与 Holm / Kuikly 无关**。

## 模块

| 模块 | Maven | 说明 |
| --- | --- | --- |
| `kmp-wgpu` | `io.github.sofarnobug:kmp-wgpu:0.1.0` | 原语 + headless Compute |
| `kmp-wgpu-surface` | `io.github.sofarnobug:kmp-wgpu-surface:0.1.0` | Surface + **FullscreenPass** |
| `kmp-wgpu-native` | Pod `KmpWgpuNative` / ohpm+npm `@jlj/kmp-wgpu-native` | Rust `.so` / wasm / xcframework |

## 消费

```kotlin
implementation("io.github.sofarnobug:kmp-wgpu:0.1.0")
implementation("io.github.sofarnobug:kmp-wgpu-surface:0.1.0")
```

Holm 联调：`KuiklyHolm/local.properties` 设 `kmp.wgpu.local=true`：
- 主 settings（Android/iOS/H5）→ `includeBuild` 本仓根
- `settings.ohos` → `includeBuild` **`ohosInclude/`**（KBA + `ohosArm64`；主 settings 无该变体）

## 发布

```bash
./publish-maven.sh # Maven Central（需凭证）
./publish-maven.sh local # mavenLocal 狗粮
```

Native：`kmp-wgpu-native/build-android.sh` / `build-ios.sh` / `build-ohos.sh` / `build-wasm.ps1`。

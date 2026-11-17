# kmp-wgpu-native · kmp_wgpu_native

> **长期身份**：`kmp_wgpu_native`（Kotlin 脸：`:kmp-wgpu` / `:kmp-wgpu-surface`）。 
> **产物文件名**为 `libkmp_wgpu_native.so` / `kmp_wgpu_native*.wasm` / xcframework。 
> **内部**：`primitives` + `fullscreen_pass`（无 liquid/scene/ClearTri）；JNI 进 `AndroidWgpuPlatform`。 
> **状态**：H5 wasm + Android JNI + iOS xcframework + 鸿蒙 `.so` 已通。  
> 尖刺默认：**Rust + wgpu 30.0.2**；禁止静默换非 wgpu 栈。

## 范围

- H5：`KmpWgpuSession` + `onReady.backend=wgpu-m0`（或 `wgpu-m0-gl`）
- Android：JNI → `AndroidWgpuPlatform` + `SurfaceView` 出画
- iOS：`kmp_wgpu_session_*` C ABI + `CAMetalLayer`；宿主外泵 tick
- 鸿蒙：`kmp_wgpu_session_create_ohos` + `libkmp_wgpu_native.so`；宿主 XComponent OnFrame 外泵
- Present：`FullscreenPass.setFragmentShader` 热换片元；无片元时只清屏（无 liquid/scene）
- **不做（本波）**：逐字 GLSL≡WGSL、真 Backdrop、改 O0、Rust 内泵、Android 多 ABI、把 Android JNI 收拢到 C Session

## H5 解锁

```powershell
cd kmp-wgpu-native
.\build-wasm.ps1
# → h5App/src/jsMain/resources/kmp-wgpu-native/
# 再编 h5App + holmDemo pack，起 8080/8083，进 Runtime 旁路页切三席
```

`wasm-bindgen-cli` 须与 crate 锁定的 `wasm-bindgen` **精确一致**（当前 0.2.130）。

## Android 解锁

```powershell
# 需要：ANDROID_NDK_HOME、rustup target aarch64-linux-android、cargo-ndk
cd kmp-wgpu-native
.\build-android.ps1
# → androidApp/src/main/jniLibs/arm64-v8a/libkmp_wgpu_native.so
```

```bash
# macOS / Linux 等价
./build-android.sh
```

- release：`opt-level=s` + lto + strip + codegen-units=1（见 Cargo.toml）；arm64 `.so` ≈ 4.0 MiB 
- JNI 类：**`com.jlj.kmp.wgpu.AndroidWgpuPlatform`**（kmp-wgpu v1；勿再链 `HolmWgpuNative` native） 
- 改 Runtime Rust 后须先跑脚本；Gradle **不**自动编 Rust。缺 `.so` → Demo `onError(BLOCKED)`，不静默 GLES。

## iOS 解锁

```bash
# 需要：rustup（aarch64-apple-ios + aarch64-apple-ios-sim）、Xcode
cd kmp-wgpu-native
./build-ios.sh
# → dist/ios/KmpWgpuNative.xcframework + Headers/kmp_wgpu_native.h
# iosApp 直链该 xcframework；改 Rust 后须重跑本脚本（Xcode 不自动编 Rust）
```

缺 xcframework → 工程链接失败；create/tick 失败 → Demo `onError(BLOCKED)`，禁止 GLES 冒充。

## 鸿蒙解锁

```bash
# 需要：rustup target aarch64-unknown-linux-ohos、OpenHarmony native SDK
# 可选环境变量：OHOS_SDK_NATIVE / OHOS_NDK_HOME / DEVECO_SDK_HOME
cd kmp-wgpu-native
./build-ohos.sh
# → ohpm/src/main/ + ohosApp/entry/libs/arm64-v8a/libkmp_wgpu_native.so
# 改 Rust 后须重跑本脚本（Hvigor 不编 Rust）
```

缺 `.so` / create 失败 → 宿主应诚实报错，禁止用 GLES 冒充 wgpu。

## 验收

- [x] 工具链 + wasm-bindgen
- [x] H5 Session 泵帧
- [x] H5 `setFragmentShader` + 空片元只清屏
- [x] Android arm64 `.so` + JNI Session API
- [ ] Android 真机 FullscreenPass 自定义片元
- [x] iOS device+sim xcframework
- [x] 鸿蒙 aarch64 `libkmp_wgpu_native.so`
- [ ] resize / destroy 专项压测
- [x] H5 wasm 包体 ≈ 4.02 MiB（wgpu 30）

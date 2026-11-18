# 开发者文档 · kmp-wgpu

给**维护 / 共建本仓**的开发者。  
路人看根 [README.md](README.md)；接入工程看 [docs/guide.md](docs/guide.md)。

## 仓库布局

| 路径 | 职责 |
|---|---|
| `kmp-wgpu/` | KMP 原语（Headless Compute、Runtime / Device 壳） |
| `kmp-wgpu-surface/` | 可选 Present · `FullscreenPass` · `*Surfaces` |
| `kmp-wgpu-native/` | Rust + wgpu 30 · 各端 `.so` / xcframework / wasm |
| `ohosInclude/` | 下游 Ohos `includeBuild` 用的 KBA composite（主 settings 无 `ohosArm64`） |
| `sample/` | 独立冒烟说明 |
| `docs/guide.md` | 接入手册 |

本仓是跨端 **wgpu 原语层**：通用算力与 Present 粗块。不要把上层业务 Session / 特效目录塞回本仓。

## 本地构建 Native

改 Rust 后 **必须**重跑对应脚本（Gradle / Xcode / Hvigor **不会**自动编 Rust）：

```bash
cd kmp-wgpu-native
./build-android.sh    # → jniLibs / 下游拷贝路径见脚本
./build-ios.sh        # → dist/ios/KmpWgpuNative.xcframework（headers 与 .a 分目录，避免体积翻倍）
./build-ohos.sh       # → ohpm/src/main + ohosApp libs
# H5（Windows 示例）
.\build-wasm.ps1
```

工具链要点：

- Android：`ANDROID_NDK_HOME`、`aarch64-linux-android`、`cargo-ndk`
- iOS：`aarch64-apple-ios` + sim、Xcode
- 鸿蒙：`aarch64-unknown-linux-ohos`、OpenHarmony native SDK（`OHOS_SDK_NATIVE` 等）
- wasm：`wasm-bindgen-cli` 须与 crate 锁定的 `wasm-bindgen` **精确一致**

## 发布

**0.1.0 渠道状态**见根 [CHANGELOG.md](CHANGELOG.md)。发版前先打 **同版本 git tag**（CocoaPods `s.source` 按 tag 拉仓）。

### Maven Central

```bash
./publish-maven.sh          # Central（需凭证）
./publish-maven.sh local    # mavenLocal 狗粮
```

坐标：`io.github.sofarnobug:kmp-wgpu` / `kmp-wgpu-surface`。  
**0.1.0 已发 Central。**

### 鸿蒙 ohpm

```bash
cd kmp-wgpu-native/ohpm
./publish-ohos.sh
```

口令经 hideable 读取时，**整段 `send "pass\r"` 会因控制字符 `\p{Cc}` 被拒**，上层常误报 `key_path error`。脚本用 `_publish.expect` **逐字符**喂入口令。详见脚本头注释。  
**0.1.1 已过审上架**（`ohpm info @jlj/kmp-wgpu-native` → `latest: 0.1.1`）。

### iOS CocoaPods

- podspec：**仓根** `KmpWgpuNative.podspec`（路径相对仓库根）
- xcframework：`build-ios.sh` → 入库 `kmp-wgpu-native/dist/ios/`（须进 git，供 tag 检出）
- iOS 部署下限 **15.0**；模拟器仅 arm64（排除 `x86_64`）
- 发布：

```bash
pod trunk push KmpWgpuNative.podspec --allow-warnings
```

**0.1.0 已上 trunk**（`pod trunk info KmpWgpuNative` / trunk API）；CDN / `pod search` 可能滞后数小时。  
注意：trunk 偶发 “internal server error” 后可能其实已入库；重推会报 `duplicate entry`，以 trunk API 为准，勿强行改版本号重发。

### H5 npm

```bash
cd kmp-wgpu-native/npm
npm login
npm publish --access public
```

包名 `@sofarnobug/kmp-wgpu-native`（npm org=`sofarnobug`；ohpm 仍为 `@jlj/...`）。**0.1.0 已发**（`npm view` → `latest: 0.1.0`）。

### GitHub Release（可选）

tag 推送后可在网页或 `gh release create 0.1.0` 补 Release 说明；与 Maven / Pod 无硬依赖。**[0.1.0 Release](https://github.com/SoFarNoBug/kmp-wgpu/releases/tag/0.1.0) 已建**。

## 下游本地联调（可选）

若下游工程（如渲染底座）用 `includeBuild` 联调本仓，可在其 `local.properties` 设：

```properties
kmp.wgpu.local=true
```

- 主 settings（Android / iOS / H5）→ `includeBuild` **本仓根**
- Ohos settings → `includeBuild` **`ohosInclude/`**（KBA + `ohosArm64`）

关闭 local 后走 Maven：`io.github.sofarnobug:kmp-wgpu*:0.1.0`。

## 文档约定

| 读者 | 文件 |
|---|---|
| 路人 | 根 `README.md` |
| 接入 | `docs/guide.md`（含公开 API 全表，对齐 Holm guide 粒度） |
| 版本 / 渠道状态 | 根 `CHANGELOG.md` |
| 开发者 / 维护 | 本文件 |

根 README 场景表用 **`已有` / `规划中` / `非目标`**；后端表以运行时 `backendId()` 为准：鸿蒙写清 GLES 实测与 Vulkan 适配缺口，Android **不写死** Vulkan。

## PR 建议

1. 改 native ABI / 符号 → 同步头文件、Kotlin 绑定、各端产物与 CHANGELOG。
2. 改示例 WGSL → 同步 Kotlin `COMPUTE_SAMPLE_DOUBLE_WGSL` 与 Rust `SAMPLE_DOUBLE_WGSL`。
3. Present / create 失败时保持诚实报错，不要用别的图形栈顶替成功。
4. 提交说明建议：`feat` / `fix` / `docs` / `refactor` / `chore` 前缀。

# kmp-wgpu-surface

可选 **Present / Surface** 模块。只要无窗口算力时，不必依赖本模块。

| 端 | 入口 |
|---|---|
| Android | `AndroidSurfaces.create` / `createFullscreenPass` |
| iOS | `IosSurfaces.create` |
| 鸿蒙 | `OhosSurfaces.create` |
| H5 | `JsSurfaces.create`（异步） |

核心类型：`GpuSurface`、`FullscreenPass`（`setFragmentShader` / `tick` / `backendId`）。

依赖 [`:kmp-wgpu`](../kmp-wgpu)。接入示例见 [docs/guide.md](../docs/guide.md)。

# kmp-wgpu-surface（ADR 0013）

可选 **Present / Surface** 模块。无窗口算力可不依赖本模块。

- Android：`AndroidSurfaces.create(android.view.Surface, w, h)`
- H5：`JsSurfaces.create(canvas, w, h, onSuccess, onError)`

依赖 [`:kmp-wgpu`](../kmp-wgpu)。

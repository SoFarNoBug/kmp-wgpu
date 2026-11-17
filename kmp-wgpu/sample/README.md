# kmp-wgpu sample（无 Holm / Kuikly）

ADR 0015 · 独立仓冒烟入口。

## Headless Compute

```kotlin
import com.jlj.kmp.wgpu.ComputeReadbackSample

// Android：见 ComputeReadbackSample / createHeadlessDeviceAsync
// 期望：FloatArray(8) { it * 2f } 量级结果（×2 WGSL）
```

## Present（FullscreenPass）

1. 用 `AndroidSurfaces.create(surface, w, h)` / `JsSurfaces.create(...)` 拿到 `GpuSurface`
2. `FullscreenPass(gpu).setFragmentShader(solidFs)`
3. 每帧 `tick(dt)`

示例片元（solid）：

```wgsl
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
    return vec4f(0.2, 0.55, 0.9, 1.0);
}
```

零 Kuikly / Holm 依赖；验收入口见独立仓 `sample/`。

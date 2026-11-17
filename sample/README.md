# kmp-wgpu sample

独立仓冒烟入口（不依赖上层业务壳）。

## Headless Compute

```kotlin
import com.jlj.kmp.wgpu.ComputeReadbackSample

// 见 ComputeReadbackSample / createHeadlessDeviceAsync
// 期望：输入 ×2（示例 WGSL）
ComputeReadbackSample.run { result ->
    result.onSuccess { /* … */ }
}
```

## Present（FullscreenPass）

1. `AndroidSurfaces.create(surface, w, h)` / `JsSurfaces.create(...)` → `GpuSurface`
2. `FullscreenPass(gpu).setFragmentShader(solidFs)`
3. 每帧 `tick(dt)`

示例片元：

```wgsl
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
    return vec4f(0.2, 0.55, 0.9, 1.0);
}
```

更多接入说明：[docs/guide.md](../docs/guide.md)。

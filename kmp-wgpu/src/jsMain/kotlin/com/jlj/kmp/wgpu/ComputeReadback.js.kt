package com.jlj.kmp.wgpu

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: H5 headless Compute + 回读 → __kmpWgpu.createHeadless / runF32（异步）。
 */

actual fun createHeadlessDevice(): HeadlessDevice? = null

actual fun createHeadlessDeviceAsync(onDone: (Result<HeadlessDevice>) -> Unit) {
    if (!WgpuRuntime.isAvailable()) {
        onDone(Result.failure(IllegalStateException(WgpuRuntime.blockedMessage())))
        return
    }
    val done: (Any?, String?) -> Unit = { session, err ->
        if (err != null || session == null) {
            onDone(Result.failure(IllegalStateException(err ?: "createHeadless failed")))
        } else {
            onDone(
                Result.success(
                    HeadlessDevice(session) { h ->
                        js(
                            """
                            (function(s){
                              if (!s) return;
                              if (typeof s.destroy === 'function') s.destroy();
                              else if (typeof s.free === 'function') s.free();
                            })
                            """,
                        ).unsafeCast<(Any?) -> Unit>()(h)
                    },
                ),
            )
        }
    }
    try {
        js(
            """
            (function(done){
              var rt = window.__kmpWgpu;
              if (!rt || typeof rt.createHeadless !== 'function') {
                done(null, 'createHeadless missing on __kmpWgpu');
                return;
              }
              Promise.resolve(rt.createHeadless())
                .then(function(s){ done(s, null); })
                .catch(function(e){
                  done(null, (e && e.message) ? e.message : String(e));
                });
            })
            """,
        ).unsafeCast<((Any?, String?) -> Unit) -> Unit>()(done)
    } catch (t: Throwable) {
        onDone(Result.failure(t))
    }
}

actual fun HeadlessDevice.computeReadback(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
): Result<FloatArray> {
    return Result.failure(
        UnsupportedOperationException("H5 computeReadback is async; use computeReadbackAsync"),
    )
}

actual fun HeadlessDevice.computeReadbackAsync(
    shaderWgsl: String,
    input: FloatArray,
    workgroupCountX: Int,
    onDone: (Result<FloatArray>) -> Unit,
) {
    if (!isAlive) {
        onDone(Result.failure(IllegalStateException("headless device destroyed")))
        return
    }
    if (input.isEmpty()) {
        onDone(Result.failure(IllegalArgumentException("empty input")))
        return
    }
    val session = this.handle
    val wg = workgroupCountX.coerceAtLeast(1)
    val done: (Any?, String?) -> Unit = { out, err ->
        if (err != null || out == null) {
            onDone(Result.failure(IllegalStateException(err ?: "runF32 failed")))
        } else {
            try {
                onDone(Result.success(float32ToKotlin(out)))
            } catch (t: Throwable) {
                onDone(Result.failure(t))
            }
        }
    }
    try {
        js(
            """
            (function(session, wgsl, input, wg, done){
              if (!session || typeof session.runF32 !== 'function') {
                done(null, 'runF32 missing on HeadlessComputeSession');
                return;
              }
              var arr = new Float32Array(input.length);
              for (var i = 0; i < input.length; i++) arr[i] = input[i];
              Promise.resolve(session.runF32(wgsl, arr, wg))
                .then(function(out){ done(out, null); })
                .catch(function(e){
                  done(null, (e && e.message) ? e.message : String(e));
                });
            })
            """,
        ).unsafeCast<(Any?, String, FloatArray, Int, (Any?, String?) -> Unit) -> Unit>()(
            session,
            shaderWgsl,
            input,
            wg,
            done,
        )
    } catch (t: Throwable) {
        onDone(Result.failure(t))
    }
}

private fun float32ToKotlin(arr: Any): FloatArray {
    val len = js("(function(a){ return a.length|0; })").unsafeCast<(Any) -> Int>()(arr)
    val out = FloatArray(len)
    for (i in 0 until len) {
        out[i] = js("(function(a,i){ return +a[i]; })").unsafeCast<(Any, Int) -> Double>()(arr, i).toFloat()
    }
    return out
}

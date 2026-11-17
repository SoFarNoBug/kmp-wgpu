package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device
import com.jlj.kmp.wgpu.WgpuRuntime
import com.jlj.kmp.wgpu.jsPlatform
import org.w3c.dom.HTMLCanvasElement

/**
 * @CreateDate: 2026/11/16
 * @Author: jlj
 * @Description: H5 canvas → Device + GpuSurface（异步 createSession）。
 */

object JsSurfaces {
    fun create(
        canvas: HTMLCanvasElement,
        width: Int,
        height: Int,
        onSuccess: (GpuSurface) -> Unit,
        onError: (String) -> Unit,
    ) {
        if (!WgpuRuntime.isAvailable()) {
            onError(WgpuRuntime.blockedMessage())
            return
        }
        val w = width.coerceAtLeast(1)
        val h = height.coerceAtLeast(1)
        val ok = { session: dynamic, _: String ->
            val device = Device.wrap(session as Any, jsPlatform())
            onSuccess(
                GpuSurface(
                    device = device,
                    initialConfig = SurfaceConfiguration(w, h),
                ),
            )
        }
        val err = { message: String -> onError(message) }
        js(
            """
            (function(canvas, w, h, ok, err) {
              var rt = window.__kmpWgpu;
              if (!rt || typeof rt.createSession !== 'function') {
                err('kmp-wgpu-native createSession missing');
                return;
              }
              canvas.width = Math.max(1, w | 0);
              canvas.height = Math.max(1, h | 0);
              Promise.resolve(rt.createSession(canvas)).then(
                function(session) {
                  var backend = 'wgpu-m0';
                  try {
                    if (session && typeof session.backendId === 'function') {
                      backend = String(session.backendId() || backend);
                    }
                  } catch (e) {}
                  ok(session, backend);
                },
                function(e) {
                  var msg = (e && e.message) ? e.message : String(e);
                  err(msg);
                }
              );
            })
            """,
        ).unsafeCast<
            (
                HTMLCanvasElement,
                Int,
                Int,
                (dynamic, String) -> Unit,
                (String) -> Unit,
            ) -> Unit,
            >()(canvas, w, h, ok, err)
    }
}

internal actual fun resizePlatform(device: Device, width: Int, height: Int) {
    js(
        "(function(s,w,h){ if (s && typeof s.resize === 'function') s.resize(w,h); })",
    ).unsafeCast<(Any?, Int, Int) -> Unit>()(device.handle, width.coerceAtLeast(1), height.coerceAtLeast(1))
}

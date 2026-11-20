package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.WgpuRuntime
import org.w3c.dom.HTMLCanvasElement

/**
 * @CreateDate: 2026/11/19
 * @Author: jlj
 * @Description: H5 ModelPass → wasm KmpWgpuModelSession（仅 WebGPU；无则 create 失败）。
 */
class ModelPass private constructor(
    private var _session: dynamic,
) {
    val handle: dynamic get() = _session

    fun resize(width: Int, height: Int) {
        val s = _session ?: return
        js("(function(s,w,h){ if (s && typeof s.resize === 'function') s.resize(w,h); })")
            .unsafeCast<(dynamic, Int, Int) -> Unit>()(s, width.coerceAtLeast(1), height.coerceAtLeast(1))
    }

    fun setMeshes(verts: FloatArray, indices: IntArray): Boolean {
        val s = _session ?: return false
        return try {
            js(
                """
                (function(s, verts, indices){
                  if (!s || typeof s.setMeshes !== 'function') return false;
                  var v = (verts && verts.length != null) ? verts : new Float32Array(0);
                  var idx = new Uint32Array(indices.length);
                  for (var i = 0; i < indices.length; i++) idx[i] = indices[i] >>> 0;
                  s.setMeshes(v, idx);
                  return true;
                })
                """,
            ).unsafeCast<(dynamic, FloatArray, IntArray) -> Boolean>()(s, verts, indices)
        } catch (_: Throwable) {
            false
        }
    }

    fun setFrame(mvp: FloatArray, color: FloatArray, lit: Boolean, lightDir: FloatArray) {
        val s = _session ?: return
        js(
            """
            (function(s, mvp, color, lit, lightDir){
              if (!s || typeof s.setFrame !== 'function') return;
              s.setFrame(mvp, color, !!lit, lightDir);
            })
            """,
        ).unsafeCast<(dynamic, FloatArray, FloatArray, Boolean, FloatArray) -> Unit>()(
            s, mvp, color, lit, lightDir,
        )
    }

    fun setPlaying(playing: Boolean) {
        val s = _session ?: return
        js("(function(s,p){ if (s && typeof s.setPlaying === 'function') s.setPlaying(!!p); })")
            .unsafeCast<(dynamic, Boolean) -> Unit>()(s, playing)
    }

    fun tick(dtSecs: Float): Boolean {
        val s = _session ?: return false
        return try {
            js(
                """
                (function(s, dt){
                  if (!s || typeof s.tick !== 'function') return false;
                  s.tick(dt);
                  return true;
                })
                """,
            ).unsafeCast<(dynamic, Float) -> Boolean>()(s, dtSecs)
        } catch (_: Throwable) {
            false
        }
    }

    fun backendId(): String {
        val s = _session ?: return "wgpu-blocked"
        return try {
            js(
                """
                (function(s){
                  if (s && typeof s.backendId === 'function') return String(s.backendId() || 'wgpu-m0');
                  return 'wgpu-m0';
                })
                """,
            ).unsafeCast<(dynamic) -> String>()(s)
        } catch (_: Throwable) {
            "wgpu-m0"
        }
    }

    fun lastErrorCode(): String {
        val s = _session ?: return ""
        return try {
            js("(function(s){ return (s && typeof s.lastErrorCode === 'function') ? String(s.lastErrorCode()||'') : ''; })")
                .unsafeCast<(dynamic) -> String>()(s)
        } catch (_: Throwable) {
            ""
        }
    }

    fun lastErrorMessage(): String {
        val s = _session ?: return ""
        return try {
            js("(function(s){ return (s && typeof s.lastErrorMessage === 'function') ? String(s.lastErrorMessage()||'') : ''; })")
                .unsafeCast<(dynamic) -> String>()(s)
        } catch (_: Throwable) {
            ""
        }
    }

    fun drainErrors(): List<Pair<String, String>> {
        val s = _session ?: return emptyList()
        return try {
            val flat = js(
                """
                (function(s){
                  if (!s || typeof s.drainErrors !== 'function') return [];
                  var arr = s.drainErrors();
                  if (!arr) return [];
                  var out = [];
                  for (var i = 0; i < arr.length; i++) {
                    var item = arr[i];
                    if (item && typeof item === 'object') {
                      out.push(String(item.code || ''));
                      out.push(String(item.message || ''));
                    }
                  }
                  return out;
                })
                """,
            ).unsafeCast<(dynamic) -> Array<String>>()(s)
            val out = ArrayList<Pair<String, String>>(flat.size / 2)
            var i = 0
            while (i + 1 < flat.size) {
                out.add(flat[i] to flat[i + 1])
                i += 2
            }
            out
        } catch (_: Throwable) {
            emptyList()
        }
    }

    fun destroy() {
        val s = _session
        _session = null
        if (s != null) {
            js("(function(s){ if (s && typeof s.free === 'function') s.free(); })")
                .unsafeCast<(dynamic) -> Unit>()(s)
        }
    }

    companion object {
        fun create(
            canvas: HTMLCanvasElement,
            width: Int,
            height: Int,
            onSuccess: (ModelPass) -> Unit,
            onError: (String) -> Unit,
        ) {
            if (!WgpuRuntime.isAvailable()) {
                onError(WgpuRuntime.blockedMessage())
                return
            }
            val w = width.coerceAtLeast(1)
            val h = height.coerceAtLeast(1)
            val ok = { session: dynamic -> onSuccess(ModelPass(session)) }
            val err = { message: String -> onError(message) }
            js(
                """
                (function(canvas, w, h, ok, err) {
                  var rt = window.__kmpWgpu;
                  if (!rt || typeof rt.createModelSession !== 'function') {
                    err('kmp-wgpu-native createModelSession missing (rebuild wasm)');
                    return;
                  }
                  canvas.width = Math.max(1, w | 0);
                  canvas.height = Math.max(1, h | 0);
                  Promise.resolve(rt.createModelSession(canvas)).then(
                    function(session) { ok(session); },
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
                    (dynamic) -> Unit,
                    (String) -> Unit,
                ) -> Unit,
                >()(canvas, w, h, ok, err)
        }
    }
}

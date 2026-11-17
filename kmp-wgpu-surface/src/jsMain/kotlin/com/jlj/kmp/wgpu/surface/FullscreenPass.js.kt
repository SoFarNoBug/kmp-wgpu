package com.jlj.kmp.wgpu.surface

import com.jlj.kmp.wgpu.Device

/**
 * @CreateDate: 2026/11/17
 * @Author: jlj
 * @Description: H5 FullscreenPass → wasm setFragmentShader / tick。
 */

internal actual fun platformSetFragmentShader(device: Device, wgslFs: String): Boolean {
    return try {
        js(
            """
            (function(s, fs){
              if (!s || typeof s.setFragmentShader !== 'function') return false;
              s.setFragmentShader(fs || '');
              return true;
            })
            """,
        ).unsafeCast<(Any?, String) -> Boolean>()(device.handle, wgslFs)
    } catch (_: Throwable) {
        false
    }
}

internal actual fun platformSetPlaying(device: Device, playing: Boolean) {
    js(
        "(function(s, p){ if (s && typeof s.setPlaying === 'function') s.setPlaying(!!p); })",
    ).unsafeCast<(Any?, Boolean) -> Unit>()(device.handle, playing)
}

internal actual fun platformTick(device: Device, dtSecs: Float): Boolean {
    return try {
        js(
            """
            (function(s, dt){
              if (!s || typeof s.tick !== 'function') return false;
              s.tick(dt);
              return true;
            })
            """,
        ).unsafeCast<(Any?, Float) -> Boolean>()(device.handle, dtSecs)
    } catch (_: Throwable) {
        false
    }
}

internal actual fun platformBackendId(device: Device): String {
    return try {
        js(
            """
            (function(s){
              if (s && typeof s.backendId === 'function') return String(s.backendId() || 'wgpu-m0');
              return 'wgpu-m0';
            })
            """,
        ).unsafeCast<(Any?) -> String>()(device.handle)
    } catch (_: Throwable) {
        "wgpu-m0"
    }
}

internal actual fun platformLastErrorCode(device: Device): String {
    return try {
        js(
            "(function(s){ return (s && typeof s.lastErrorCode === 'function') ? String(s.lastErrorCode()||'') : ''; })",
        ).unsafeCast<(Any?) -> String>()(device.handle)
    } catch (_: Throwable) {
        ""
    }
}

internal actual fun platformLastErrorMessage(device: Device): String {
    return try {
        js(
            "(function(s){ return (s && typeof s.lastErrorMessage === 'function') ? String(s.lastErrorMessage()||'') : ''; })",
        ).unsafeCast<(Any?) -> String>()(device.handle)
    } catch (_: Throwable) {
        ""
    }
}

internal actual fun platformDrainErrors(device: Device): List<Pair<String, String>> {
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
                } else {
                  out.push(String(item));
                }
              }
              return out;
            })
            """,
        ).unsafeCast<(Any?) -> Array<String>>()(device.handle)
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

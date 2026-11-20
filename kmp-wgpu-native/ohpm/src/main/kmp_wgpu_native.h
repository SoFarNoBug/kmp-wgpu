#ifndef KMP_WGPU_NATIVE_H
#define KMP_WGPU_NATIVE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** ABI version; bump when C Session layout/semantics break. */
uint32_t kmp_wgpu_abi_version(void);

/** 1 if wgpu backend compiled in; 0 otherwise. */
uint32_t kmp_wgpu_has_wgpu(void);

/**
 * Create a Present Session bound to a CAMetalLayer (iOS).
 * @param metal_layer  CAMetalLayer* (not retained by Rust; must outlive the session)
 * @param width        pixel width (>=1)
 * @param height       pixel height (>=1)
 * @return opaque handle, or 0 on failure
 */
uint64_t kmp_wgpu_session_create_metal(void *metal_layer, uint32_t width, uint32_t height);

/**
 * Create a Present Session bound to an OHNativeWindow (HarmonyOS / ADR 0010).
 * @param native_window  OHNativeWindow* from XComponent OnSurfaceCreated (must outlive session)
 * @param width          pixel width (>=1)
 * @param height         pixel height (>=1)
 * @return opaque handle, or 0 on failure
 */
uint64_t kmp_wgpu_session_create_ohos(void *native_window, uint32_t width, uint32_t height);

/**
 * Last create_metal / create_ohos / create_android failure reason (NUL-terminated UTF-8).
 * Empty if the last create succeeded. Independent of session handle.
 */
uint32_t kmp_wgpu_last_create_error(char *out_buf, uint32_t out_len);

void kmp_wgpu_session_destroy(uint64_t handle);

void kmp_wgpu_session_resize(uint64_t handle, uint32_t width, uint32_t height);

/**
 * Set WGSL fragment body (shell provides vs_main + uniforms).
 * Empty / NULL clears fragment; tick then clear-only.
 * @return 1 on success, 0 on failure
 */
uint32_t kmp_wgpu_session_set_fragment_shader(uint64_t handle, const char *fs);

void kmp_wgpu_session_set_playing(uint64_t handle, uint32_t playing);

/**
 * Advance one frame.
 * @return 1 on success, 0 on failure / invalid handle
 */
uint32_t kmp_wgpu_session_tick(uint64_t handle, float dt_secs);

/**
 * Copy backend id into out_buf (NUL-terminated). Truncates if needed.
 * @return bytes written excluding NUL, or 0
 */
uint32_t kmp_wgpu_session_backend_id(uint64_t handle, char *out_buf, uint32_t out_len);

/**
 * Last error code / message after failed set_fragment_shader / tick / uncaptured wgpu.
 * Copies NUL-terminated UTF-8; truncates if needed.
 * @return bytes written excluding NUL, or 0
 */
uint32_t kmp_wgpu_session_last_error_code(uint64_t handle, char *out_buf, uint32_t out_len);
uint32_t kmp_wgpu_session_last_error_message(uint64_t handle, char *out_buf, uint32_t out_len);

/**
 * Headless Compute（ADR 0013 v1.1）：无 Surface；与 Present Session 分表。
 * @return opaque handle, or 0 on failure（文案见 kmp_wgpu_last_create_error）
 */
uint64_t kmp_wgpu_headless_create(void);

void kmp_wgpu_headless_destroy(uint64_t handle);

uint32_t kmp_wgpu_headless_backend_id(uint64_t handle, char *out_buf, uint32_t out_len);

/**
 * WGSL compute + f32 回读。
 * @param out_floats 调用方缓冲；out_cap 为可写 float 个数（通常 = input_len）
 * @return 写入的 float 个数；0 = 失败（文案见 kmp_wgpu_last_create_error）
 */
uint32_t kmp_wgpu_headless_compute_readback(
    uint64_t handle,
    const char *wgsl,
    const float *input,
    uint32_t input_len,
    uint32_t workgroup_count_x,
    float *out_floats,
    uint32_t out_cap);

/**
 * ModelPass — mesh Present Session（与 Fullscreen Session / Headless 分表）。
 * Create bound to CAMetalLayer (iOS) or OHNativeWindow (Ohos). Surface not retained; must outlive handle.
 * Failures: handle 0 + kmp_wgpu_last_create_error.
 */
uint64_t kmp_wgpu_model_create_metal(void *metal_layer, uint32_t width, uint32_t height);
uint64_t kmp_wgpu_model_create_ohos(void *native_window, uint32_t width, uint32_t height);

void kmp_wgpu_model_destroy(uint64_t handle);

void kmp_wgpu_model_resize(uint64_t handle, uint32_t width, uint32_t height);

/**
 * Upload interleaved pos3+nrm3 verts (vert_floats % 6 == 0) and uint32 indices
 * (passed as int32_t*; cast to u32). Empty verts or indices clears mesh.
 * @return 1 on success, 0 on failure / bad handle
 */
uint32_t kmp_wgpu_model_set_meshes(
    uint64_t handle,
    const float *verts,
    uint32_t vert_floats,
    const int32_t *indices,
    uint32_t index_count);

/**
 * Per-frame uniforms: column-major mvp16, rgba color4, lit flag, light_dir xyz.
 * Null / short pointers are no-ops (same guard style as Android JNI).
 */
void kmp_wgpu_model_set_frame(
    uint64_t handle,
    const float *mvp16,
    const float *color4,
    uint32_t lit,
    const float *light_dir3);

void kmp_wgpu_model_set_playing(uint64_t handle, uint32_t playing);

/**
 * Advance one frame.
 * @return 1 on success, 0 on failure / invalid handle
 */
uint32_t kmp_wgpu_model_tick(uint64_t handle, float dt_secs);

uint32_t kmp_wgpu_model_backend_id(uint64_t handle, char *out_buf, uint32_t out_len);
uint32_t kmp_wgpu_model_last_error_code(uint64_t handle, char *out_buf, uint32_t out_len);
uint32_t kmp_wgpu_model_last_error_message(uint64_t handle, char *out_buf, uint32_t out_len);

#ifdef __cplusplus
}
#endif

#endif /* KMP_WGPU_NATIVE_H */

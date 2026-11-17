/**
 * @jlj/kmp-wgpu-native — ship libkmp_wgpu_native.so + kmp_wgpu_native.h under src/main/.
 * Consumers link the .so from this package; Kotlin cinterop uses the header.
 */
export const KMP_WGPU_NATIVE_LIB = 'kmp_wgpu_native';

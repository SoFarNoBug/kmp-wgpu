#!/usr/bin/env bash
# DualTrack B · build libkmp_wgpu_native.so (arm64-v8a) into androidApp/jniLibs
# Requires: rustup, Android NDK, cargo-ndk
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
JNI_LIBS_ROOT="$ROOT/../androidApp/src/main/jniLibs"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -z "${ANDROID_NDK_HOME}" ]]; then
  # macOS 常见本机路径
  CANDIDATE="$HOME/Library/Android/sdk/ndk"
  if [[ -d "$CANDIDATE" ]]; then
    ANDROID_NDK_HOME="$(ls -d "$CANDIDATE"/* 2>/dev/null | sort -V | tail -1)"
    export ANDROID_NDK_HOME
  fi
fi
if [[ -z "${ANDROID_NDK_HOME}" || ! -d "${ANDROID_NDK_HOME}" ]]; then
  echo "ANDROID_NDK_HOME not set / missing" >&2
  exit 1
fi
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export PATH="$HOME/.cargo/bin:$PATH"
cd "$ROOT"
rustup target add aarch64-linux-android >/dev/null
mkdir -p "$JNI_LIBS_ROOT"
cargo ndk -t arm64-v8a -o "$JNI_LIBS_ROOT" build --release
SO="$JNI_LIBS_ROOT/arm64-v8a/libkmp_wgpu_native.so"
test -f "$SO"
echo "ok: $SO ($(wc -c < "$SO") bytes)"
echo "note: JNI class = com.jlj.kmp.wgpu.AndroidWgpuPlatform (kmp-wgpu v1)"

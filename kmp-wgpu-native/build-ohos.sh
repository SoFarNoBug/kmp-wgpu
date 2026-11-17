#!/usr/bin/env bash
# DualTrack B · build libkmp_wgpu_native.so (aarch64-ohos) into ohosApp entry libs
# Requires: rustup target aarch64-unknown-linux-ohos + OpenHarmony native SDK
# Env (first hit wins): OHOS_SDK_NATIVE | OHOS_NDK_HOME | DEVECO_SDK_HOME/default/openharmony/native
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
DIST="$ROOT/dist/ohos"
HEADER="$ROOT/include/kmp_wgpu_native.h"
TARGET="aarch64-unknown-linux-ohos"
LIB_NAME="libkmp_wgpu_native.so"
# Harmony entry CMake uses OHOS_ARCH (arm64-v8a)
APP_LIBS="$ROOT/../ohosApp/entry/libs/arm64-v8a"

export PATH="${HOME}/.cargo/bin:${PATH}"
# shellcheck disable=SC1090
[ -f "${HOME}/.cargo/env" ] && source "${HOME}/.cargo/env"

resolve_native() {
  if [ -n "${OHOS_SDK_NATIVE:-}" ] && [ -d "$OHOS_SDK_NATIVE" ]; then
    echo "$OHOS_SDK_NATIVE"
    return
  fi
  if [ -n "${OHOS_NDK_HOME:-}" ] && [ -d "$OHOS_NDK_HOME" ]; then
    echo "$OHOS_NDK_HOME"
    return
  fi
  if [ -n "${DEVECO_SDK_HOME:-}" ] && [ -d "$DEVECO_SDK_HOME/default/openharmony/native" ]; then
    echo "$DEVECO_SDK_HOME/default/openharmony/native"
    return
  fi
  # Common local installs
  for cand in \
    "${HOME}/Library/OpenHarmony/Sdk/15/native" \
    "${HOME}/Library/OpenHarmony/Sdk/14/native" \
    "/Applications/DevEco-Studio.app/Contents/sdk/default/openharmony/native"
  do
    if [ -d "$cand" ]; then
      echo "$cand"
      return
    fi
  done
  return 1
}

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found; install rustup first" >&2
  exit 1
fi
if [ ! -f "$HEADER" ]; then
  echo "missing header: $HEADER" >&2
  exit 1
fi

NATIVE="$(resolve_native)" || {
  echo "OpenHarmony native SDK not found. Set OHOS_SDK_NATIVE or install DevEco SDK." >&2
  exit 1
}

CLANG="$NATIVE/llvm/bin/aarch64-unknown-linux-ohos-clang"
AR_BIN="$NATIVE/llvm/bin/llvm-ar"
SYSROOT="$NATIVE/sysroot"
if [ ! -x "$CLANG" ]; then
  echo "missing clang: $CLANG" >&2
  exit 1
fi
if [ ! -d "$SYSROOT" ]; then
  echo "missing sysroot: $SYSROOT" >&2
  exit 1
fi

rustup target add "$TARGET" >/dev/null

export CC_aarch64_unknown_linux_ohos="$CLANG"
export AR_aarch64_unknown_linux_ohos="$AR_BIN"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER="$CLANG"
# SONAME 必须是短名，否则依赖方（libholm_xc）会把绝对路径写进 NEEDED，真机加载失败
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_RUSTFLAGS="-C link-arg=--target=aarch64-linux-ohos -C link-arg=--sysroot=${SYSROOT} -C link-arg=-Wl,-soname,${LIB_NAME}"

cd "$ROOT"
echo "==> cargo build --release --target $TARGET (native=$NATIVE)"
cargo build --release --target "$TARGET"

SO_SRC="$ROOT/target/$TARGET/release/$LIB_NAME"
if [ ! -f "$SO_SRC" ]; then
  echo "missing $SO_SRC" >&2
  exit 1
fi

rm -rf "$DIST"
mkdir -p "$DIST/include" "$APP_LIBS"
cp "$SO_SRC" "$DIST/$LIB_NAME"
cp "$HEADER" "$DIST/include/kmp_wgpu_native.h"
cp "$SO_SRC" "$APP_LIBS/$LIB_NAME"
cp "$HEADER" "$APP_LIBS/kmp_wgpu_native.h"

echo "OK → $APP_LIBS/$LIB_NAME"
ls -la "$APP_LIBS/$LIB_NAME"
du -sh "$APP_LIBS/$LIB_NAME"

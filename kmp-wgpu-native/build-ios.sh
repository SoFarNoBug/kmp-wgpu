#!/usr/bin/env bash
# DualTrack B · build KmpWgpuNative.xcframework (device + sim) into dist/ios
# Requires: rustup, Xcode CLT, targets aarch64-apple-ios + aarch64-apple-ios-sim
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
DIST="$ROOT/dist/ios"
XCFRAMEWORK="$DIST/KmpWgpuNative.xcframework"
HEADER="$ROOT/include/kmp_wgpu_native.h"
LIB_NAME="libkmp_wgpu_native.a"

export PATH="${HOME}/.cargo/bin:${PATH}"
# shellcheck disable=SC1090
[ -f "${HOME}/.cargo/env" ] && source "${HOME}/.cargo/env"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found; install rustup first" >&2
  exit 1
fi
if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "xcodebuild not found; install Xcode" >&2
  exit 1
fi
if [ ! -f "$HEADER" ]; then
  echo "missing header: $HEADER" >&2
  exit 1
fi

rustup target add aarch64-apple-ios aarch64-apple-ios-sim >/dev/null

cd "$ROOT"
echo "==> cargo build --release (ios + ios-sim)"
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim

DEVICE_A="$ROOT/target/aarch64-apple-ios/release/$LIB_NAME"
SIM_A="$ROOT/target/aarch64-apple-ios-sim/release/$LIB_NAME"
if [ ! -f "$DEVICE_A" ] || [ ! -f "$SIM_A" ]; then
  echo "missing static libs:" >&2
  echo "  $DEVICE_A" >&2
  echo "  $SIM_A" >&2
  exit 1
fi

rm -rf "$DIST"
mkdir -p "$DIST/headers"
cp "$HEADER" "$DIST/headers/kmp_wgpu_native.h"

# Staging dirs for -create-xcframework (library + headers)
STAGE_DEV="$DIST/_stage/ios-arm64"
STAGE_SIM="$DIST/_stage/ios-arm64-sim"
rm -rf "$DIST/_stage"
mkdir -p "$STAGE_DEV" "$STAGE_SIM"
cp "$DEVICE_A" "$STAGE_DEV/$LIB_NAME"
cp "$SIM_A" "$STAGE_SIM/$LIB_NAME"
cp "$HEADER" "$STAGE_DEV/kmp_wgpu_native.h"
cp "$HEADER" "$STAGE_SIM/kmp_wgpu_native.h"

echo "==> xcodebuild -create-xcframework"
rm -rf "$XCFRAMEWORK"
xcodebuild -create-xcframework \
  -library "$STAGE_DEV/$LIB_NAME" -headers "$STAGE_DEV" \
  -library "$STAGE_SIM/$LIB_NAME" -headers "$STAGE_SIM" \
  -output "$XCFRAMEWORK"

rm -rf "$DIST/_stage"
# Keep a copy of the public header next to the xcframework for HEADER_SEARCH_PATHS
mkdir -p "$DIST/Headers"
cp "$HEADER" "$DIST/Headers/kmp_wgpu_native.h"

echo "OK → $XCFRAMEWORK"
ls -la "$XCFRAMEWORK"
du -sh "$XCFRAMEWORK"
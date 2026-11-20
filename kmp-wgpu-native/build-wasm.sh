#!/usr/bin/env bash
# Build kmp_wgpu_native wasm + wasm-bindgen glue (mac/linux).
# Optional: HOLM_H5_RES=/path/to/KuiklyHolm/h5App/src/jsMain/resources/kmp-wgpu-native
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
OUT_DEFAULT="$ROOT/../npm/kmp-wgpu-native"
OUT_DIR="${HOLM_H5_RES:-$OUT_DEFAULT}"
TARGET="wasm32-unknown-unknown"
WASM_IN="$ROOT/target/$TARGET/release/kmp_wgpu_native.wasm"

cd "$ROOT"
rustup target add "$TARGET" >/dev/null
cargo build --target "$TARGET" --release
if [[ ! -f "$WASM_IN" ]]; then
  echo "missing $WASM_IN" >&2
  exit 1
fi
mkdir -p "$OUT_DIR"
wasm-bindgen "$WASM_IN" \
  --target no-modules \
  --no-typescript \
  --out-dir "$OUT_DIR" \
  --out-name kmp_wgpu_native
echo "OK → $OUT_DIR"
ls -la "$OUT_DIR" | head

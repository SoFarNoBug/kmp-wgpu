#!/usr/bin/env bash
# Publish @jlj/kmp-wgpu-native to ohpm.openharmony.cn
# Passphrase: OHPM_KEY_PASSPHRASE or ~/.ssh/ohpm_jlj_new.passphrase
#
# NOTE: ohpm hideable prompt drops any stdin chunk that contains control chars
# (e.g. sending "pass\r" as one write → empty passphrase → key_path error).
# _publish.expect feeds the passphrase character-by-character.
set -euo pipefail
export COPYFILE_DISABLE=1

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

# Prefer PKCS#8 encrypted key (Node crypto + ohpm)
if [ -f "${HOME}/.ssh/ohpm_jlj_new_pkcs8.pem" ]; then
  ohpm config set key_path "${HOME}/.ssh/ohpm_jlj_new_pkcs8.pem" >/dev/null
fi

PKG_NAME=$(grep -o '"name"[[:space:]]*:[[:space:]]*"[^"]*"' oh-package.json5 | head -1 | sed 's/.*"\(.*\)".*/\1/')
PKG_VER=$(grep -o '"version"[[:space:]]*:[[:space:]]*"[^"]*"' oh-package.json5 | head -1 | sed 's/.*"\(.*\)".*/\1/')
SAFE_NAME=$(echo "$PKG_NAME" | sed 's|/|-|g')

echo "[publish-ohos] package: ${PKG_NAME}@${PKG_VER}"
echo "[publish-ohos] key_path: $(ohpm config get key_path 2>/dev/null | tr -d '\r')"

TMP=$(mktemp -d)
PKG_DIR="$TMP/package"
mkdir -p "$PKG_DIR"
cp oh-package.json5 README.md LICENSE CHANGELOG.md index.ets "$PKG_DIR/"
[ -d src ] && cp -R src "$PKG_DIR/"

HAR="$DIR/${SAFE_NAME}-${PKG_VER}.har"
tar -czf "$HAR" -C "$TMP" package
rm -rf "$TMP"

echo "[publish-ohos] packed: $HAR"

PASS="${OHPM_KEY_PASSPHRASE:-}"
if [ -z "$PASS" ] && [ -f "${HOME}/.ssh/ohpm_jlj_new.passphrase" ]; then
  PASS=$(tr -d '\r\n' < "${HOME}/.ssh/ohpm_jlj_new.passphrase")
fi
if [ -z "$PASS" ]; then
  echo "错误：需要 OHPM_KEY_PASSPHRASE 或 ~/.ssh/ohpm_jlj_new.passphrase" >&2
  exit 1
fi

PASSFILE=$(mktemp)
printf '%s' "$PASS" > "$PASSFILE"
chmod 600 "$PASSFILE"
trap 'rm -f "$PASSFILE"' EXIT

echo "[publish-ohos] -> ohpm publish (char-by-char passphrase)"
chmod +x "$DIR/_publish.expect"
"$DIR/_publish.expect" "$HAR" "$PASSFILE"

echo "[publish-ohos] done."

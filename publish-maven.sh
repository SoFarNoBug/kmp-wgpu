#!/bin/bash
# Publish kmp-wgpu + kmp-wgpu-surface to Maven Central or mavenLocal.
# Usage:
#   ./publish-maven.sh local
#   CENTRAL_USERNAME=… CENTRAL_PASSWORD=… GPG_KEY_PASSWORD=… ./publish-maven.sh
set -euo pipefail

TARGET="${1:-central}"
VERSION="${MAVEN_VERSION:-0.1.0}"
GPG_KEY_ID="${GPG_KEY_ID:-E03218FD37956AC6AE94223E041C34E7F024F7A0}"

if [ "$TARGET" = "local" ]; then
  ./gradlew :kmp-wgpu:publishToMavenLocal :kmp-wgpu-surface:publishToMavenLocal \
    -PmavenVersion="${VERSION}" \
    -PskipSign=true
  echo "mavenLocal OK: io.github.sofarnobug:kmp-wgpu:${VERSION}"
  exit 0
fi

GPG_BIN="${GPG_BIN:-$(command -v gpg || true)}"
CENTRAL_USER="${CENTRAL_USERNAME:-}"
CENTRAL_PASS="${CENTRAL_PASS:-${CENTRAL_PASSWORD:-}}"
if [ -z "$CENTRAL_USER" ] || [ -z "$CENTRAL_PASS" ] || [ -z "${GPG_KEY_PASSWORD:-}" ]; then
  echo "错误：需要 CENTRAL_USERNAME / CENTRAL_PASSWORD / GPG_KEY_PASSWORD" >&2
  exit 1
fi
SIGNING_KEY=$("$GPG_BIN" --batch --pinentry-mode loopback --passphrase "$GPG_KEY_PASSWORD" \
  --armor --export-secret-keys "$GPG_KEY_ID")
export ORG_GRADLE_PROJECT_signingInMemoryKey="$SIGNING_KEY"
export ORG_GRADLE_PROJECT_signingInMemoryKeyPassword="$GPG_KEY_PASSWORD"

./gradlew :kmp-wgpu:publishAllPublicationsToMavenCentralRepository \
  :kmp-wgpu-surface:publishAllPublicationsToMavenCentralRepository \
  -PmavenVersion="${VERSION}" \
  -PmavenCentralUsername="${CENTRAL_USER}" \
  -PmavenCentralPassword="${CENTRAL_PASS}"
echo "Maven Central OK: io.github.sofarnobug:kmp-wgpu:${VERSION}"

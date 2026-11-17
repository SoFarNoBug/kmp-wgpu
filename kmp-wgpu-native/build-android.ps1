# DualTrack B · build libkmp_wgpu_native.so (arm64-v8a) into androidApp jniLibs
# Requires: rustup, Android NDK, cargo-ndk
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
# cargo-ndk -o 会再拼 ABI 子目录，故指向 jniLibs 根
$JniLibsRoot = Join-Path $Root "..\androidApp\src\main\jniLibs"
$AbiDir = Join-Path $JniLibsRoot "arm64-v8a"

Push-Location $Root
try {
    $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
    if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
        cargo install cargo-ndk --locked
    }
    rustup target add aarch64-linux-android | Out-Null
    New-Item -ItemType Directory -Force -Path $JniLibsRoot | Out-Null
    cargo ndk -t arm64-v8a -o $JniLibsRoot build --release
    $so = Join-Path $AbiDir "libkmp_wgpu_native.so"
    if (-not (Test-Path $so)) {
        throw "missing $so"
    }
    # 清掉误嵌套的 arm64-v8a/arm64-v8a（旧脚本遗留）
    $nested = Join-Path $AbiDir "arm64-v8a"
    if (Test-Path $nested) {
        Remove-Item -Recurse -Force $nested
    }
    Write-Host "OK → $so"
    Get-Item $so | Select-Object FullName, Length
} finally {
    Pop-Location
}

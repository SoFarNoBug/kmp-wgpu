# DualTrack B · build kmp-wgpu-native wasm + wasm-bindgen glue for h5App
# Requires: rustc, wasm32-unknown-unknown, wasm-bindgen-cli (match crate lock)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$OutDir = Join-Path $Root "..\h5App\src\jsMain\resources\kmp-wgpu-native"
$WasmIn = Join-Path $Root "target\wasm32-unknown-unknown\release\kmp_wgpu_native.wasm"

Push-Location $Root
try {
    $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
    rustup target add wasm32-unknown-unknown | Out-Null
    cargo build --target wasm32-unknown-unknown --release
    if (-not (Test-Path $WasmIn)) {
        throw "missing $WasmIn"
    }
    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
    # Align CLI with crate: read wasm-bindgen version from Cargo.lock if present
    wasm-bindgen $WasmIn `
        --target no-modules `
        --no-typescript `
        --out-dir $OutDir `
        --out-name kmp_wgpu_native
    Write-Host "OK → $OutDir"
    Get-ChildItem $OutDir | Select-Object Name, Length
} finally {
    Pop-Location
}

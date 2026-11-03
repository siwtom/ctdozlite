# Build CTDO Zalo Lite (.apk) for Android (ARM64) using cargo-apk
$ErrorActionPreference = "Stop"
$env:CMAKE_POLICY_VERSION_MINIMUM = "3.5"
$env:LIBOPUS_STATIC = "1"
$env:LIBOPUS_NO_PKG = "1"

if (-not $env:ANDROID_HOME -and (Test-Path "$env:USERPROFILE\scoop\apps\android-clt\current")) {
    $env:ANDROID_HOME = "$env:USERPROFILE\scoop\apps\android-clt\current"
}
if ($env:ANDROID_HOME -and -not $env:ANDROID_NDK_ROOT) {
    $ndkDir = Get-ChildItem "$env:ANDROID_HOME\ndk" -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1
    if ($ndkDir) {
        $env:ANDROID_NDK_ROOT = $ndkDir.FullName
        $env:ANDROID_NDK_HOME = $ndkDir.FullName
        $env:CMAKE_ANDROID_NDK = $ndkDir.FullName
        $env:CMAKE_ANDROID_ARCH_ABI = "arm64-v8a"
    }
}

Write-Host "=== Building CTDO Zalo Lite Android APK (aarch64-linux-android) ===" -ForegroundColor Cyan

# 1. Ensure Rust Android target is installed
rustup target add aarch64-linux-android

# 2. Ensure cargo-apk is installed
if (-not (Get-Command cargo-apk -ErrorAction SilentlyContinue)) {
    Write-Host "Installing cargo-apk..." -ForegroundColor Yellow
    cargo install cargo-apk --locked
}

# 3. Build release APK
Push-Location "$PSScriptRoot\..\.."
try {
    cargo apk build --release --lib
    Write-Host "`n✓ APK built successfully:" -ForegroundColor Green
    Write-Host "  target\release\apk\ctdozlite.apk" -ForegroundColor Green
} finally {
    Pop-Location
}

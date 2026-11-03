#!/usr/bin/env bash
set -euo pipefail
export CMAKE_POLICY_VERSION_MINIMUM="3.5"
export LIBOPUS_STATIC="1"
export LIBOPUS_NO_PKG="1"
if [ -n "${ANDROID_NDK_ROOT:-}" ]; then
    export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
    export CMAKE_ANDROID_NDK="$ANDROID_NDK_ROOT"
    export CMAKE_ANDROID_ARCH_ABI="arm64-v8a"
fi

echo "=== Building CTDO Zalo Lite Android APK (aarch64-linux-android) ==="
rustup target add aarch64-linux-android

if ! command -v cargo-apk >/dev/null 2>&1; then
    echo "Installing cargo-apk..."
    cargo install cargo-apk --locked
fi

cd "$(dirname "$0")/../.."
cargo apk build --release --lib
echo "✓ APK built successfully: target/release/apk/ctdozlite.apk"

#!/usr/bin/env bash
# Build unsigned/ad-hoc iOS .ipa bundle for AltStore / SideStore / Sideloadly / TrollStore
# Works on both local macOS and GitHub Actions (macos-latest runner)
set -euo pipefail

cd "$(dirname "$0")/../.."
echo "=== Building CTDO Zalo Lite for iOS (aarch64-apple-ios) ==="

export CMAKE_POLICY_VERSION_MINIMUM="3.5"
export LIBOPUS_STATIC="1"
export LIBOPUS_NO_PKG="1"
export IPHONEOS_DEPLOYMENT_TARGET="15.0"
export SDKROOT="$(xcrun --sdk iphoneos --show-sdk-path)"
export CMAKE_OSX_SYSROOT="$SDKROOT"
export CMAKE_OSX_ARCHITECTURES="arm64"
export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos -f clang)"
export CXX_aarch64_apple_ios="$(xcrun --sdk iphoneos -f clang++)"
export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos -f ar)"
export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$(xcrun --sdk iphoneos -f clang)"

rustup target add aarch64-apple-ios
cargo build --release --locked --target aarch64-apple-ios --bin zalo-rust

APP_DIR="target/ios/Payload/CTDOZLite.app"
rm -rf target/ios
mkdir -p "$APP_DIR"

cp target/aarch64-apple-ios/release/zalo-rust "$APP_DIR/zalo-rust"
chmod +x "$APP_DIR/zalo-rust"
cp packaging/ios/Info.plist "$APP_DIR/Info.plist"
cp src/assets/app_icon.png "$APP_DIR/AppIcon60x60@3x.png"

# Ad-hoc sign with ldid or codesign so Sideloadly / AltStore / TrollStore can install it directly
if command -v ldid >/dev/null 2>&1; then
    ldid -S "$APP_DIR/zalo-rust"
elif command -v codesign >/dev/null 2>&1; then
    codesign --force --sign - "$APP_DIR"
fi

mkdir -p dist
cd target/ios
zip -qr ../../dist/ctdozlite-0.0.1-ios-arm64.ipa Payload
cd ../..
cp dist/ctdozlite-0.0.1-ios-arm64.ipa target/ctdozlite-ios-unsigned.ipa
echo "✓ Built iOS IPA: dist/ctdozlite-0.0.1-ios-arm64.ipa (ready for AltStore / Sideloadly / TrollStore)"


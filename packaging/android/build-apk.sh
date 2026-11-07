#!/usr/bin/env bash
# Builds horizon-android.apk (arm64) from the repository root:
#   packaging/android/build-apk.sh [--profile fast|release]
# Needs: ANDROID_HOME (platforms;android-35, build-tools;35.0.0), ANDROID_NDK_HOME,
# `rustup target add aarch64-linux-android`, `cargo install cargo-ndk`, and a JDK for keytool.
set -euo pipefail
cd "$(dirname "$0")/../.."
PROFILE=fast
[ "${1:-}" = "--profile" ] && PROFILE="$2"
: "${ANDROID_HOME:?set ANDROID_HOME to the Android SDK}"
: "${ANDROID_NDK_HOME:?set ANDROID_NDK_HOME to the NDK}"
BT="$ANDROID_HOME/build-tools/${BUILD_TOOLS:-35.0.0}"
PLATFORM="$ANDROID_HOME/platforms/android-${PLATFORM_API:-35}/android.jar"
OUT=target/android
STAGE=$OUT/stage
rm -rf "$STAGE" && mkdir -p "$STAGE/lib/arm64-v8a"

cargo ndk -t arm64-v8a -P 26 build -p horizon-android --profile "$PROFILE"
DIR=$PROFILE; [ "$PROFILE" = dev ] && DIR=debug
cp "target/aarch64-linux-android/$DIR/libhorizon_android.so" "$STAGE/lib/arm64-v8a/"
LIBCXX=$(find "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt" -path '*aarch64-linux-android/libc++_shared.so' | head -1)
[ -n "$LIBCXX" ] && cp "$LIBCXX" "$STAGE/lib/arm64-v8a/"

"$BT/aapt2" compile --dir packaging/android/res -o "$OUT/res.zip"
"$BT/aapt2" link -o "$OUT/unsigned.apk" -I "$PLATFORM" \
    --manifest packaging/android/AndroidManifest.xml "$OUT/res.zip"
(cd "$STAGE" && zip -qr -0 ../unsigned.apk lib)
"$BT/zipalign" -f -p 4 "$OUT/unsigned.apk" "$OUT/aligned.apk"

KEY=${HORIZON_ANDROID_KEYSTORE:-$OUT/debug.keystore}
if [ ! -f "$KEY" ]; then
    keytool -genkeypair -keystore "$KEY" -storepass android -keypass android -alias horizon \
        -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Horizon Android Debug" >/dev/null
fi
"$BT/apksigner" sign --ks "$KEY" --ks-pass "pass:${HORIZON_ANDROID_KEYSTORE_PASS:-android}" \
    --ks-key-alias "${HORIZON_ANDROID_KEY_ALIAS:-horizon}" --out "$OUT/horizon-android.apk" "$OUT/aligned.apk"
ls -la "$OUT/horizon-android.apk"

#!/usr/bin/env bash
set -euo pipefail

# OHOS_NDK_HOME is the OpenHarmony SDK root containing native/, as expected by napi-build-ohos.
: "${OHOS_NDK_HOME:?Set OHOS_NDK_HOME to the OpenHarmony SDK directory containing native/}"
native_dir="$OHOS_NDK_HOME/native"
if [[ ! -x "$native_dir/llvm/bin/clang" ]]; then
  echo "Cannot find SDK clang at $native_dir/llvm/bin/clang" >&2
  exit 1
fi

export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER="$native_dir/llvm/bin/clang"
export CC_aarch64_unknown_linux_ohos="$native_dir/llvm/bin/clang"
export AR_aarch64_unknown_linux_ohos="$native_dir/llvm/bin/llvm-ar"
export CFLAGS_aarch64_unknown_linux_ohos="--target=aarch64-linux-ohos --sysroot=\"$native_dir/sysroot\""
# Encoded flags preserve SDK paths containing spaces.
export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS:+${CARGO_ENCODED_RUSTFLAGS}$'\x1f'}""-C"$'\x1f'"link-arg=--target=aarch64-linux-ohos"
export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS}"$'\x1f'"-C"$'\x1f'"link-arg=--sysroot=$native_dir/sysroot"
cd "$(dirname "$0")/../.."
cargo build --locked -p hello_openharmony --target aarch64-unknown-linux-ohos --release "$@"

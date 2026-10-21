#!/bin/sh
set -eu

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [ -n "${OHOS_NATIVE_SDK_HOME:-}" ]; then
    native_sdk_root=$OHOS_NATIVE_SDK_HOME
elif [ -n "${DEVECO_SDK_HOME:-}" ]; then
    native_sdk_root=$DEVECO_SDK_HOME/default/openharmony/native
else
    echo "Set OHOS_NATIVE_SDK_HOME or DEVECO_SDK_HOME to the installed API 26 SDK." >&2
    exit 1
fi

target_linker=$native_sdk_root/llvm/bin/aarch64-unknown-linux-ohos-clang
if [ ! -x "$target_linker" ]; then
    echo "Target linker not found: $target_linker" >&2
    exit 1
fi

cd "$project_root"
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER=$target_linker \
    cargo build --locked --offline --target aarch64-unknown-linux-ohos

source_library=$project_root/target/aarch64-unknown-linux-ohos/debug/libgpui_ohos_mobile_counter.so
module_dir=$project_root/entry/libs/arm64-v8a
mkdir -p "$module_dir"
cp "$source_library" "$module_dir/libgpui_mobile_counter.so"
echo "Rust Node-API module staged in entry/libs/arm64-v8a"

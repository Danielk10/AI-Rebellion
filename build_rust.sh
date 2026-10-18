#!/bin/sh
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/app/src/main/rust"
echo "🦀 Compilando módulo nativo Rust en Termux (16 KB + PIE)..."
export CARGO_BUILD_JOBS=2
export LIBRARY_PATH="$SCRIPT_DIR/app/src/main/rust/.stubs:${LIBRARY_PATH:-}"
export CARGO_TARGET_DIR=/tmp/cargo_ai_rebellion_target
cargo build --release
DEST_DIR="$SCRIPT_DIR/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$DEST_DIR"
if [ -f /tmp/cargo_ai_rebellion_target/release/libai_rebellion.so ]; then
    cp /tmp/cargo_ai_rebellion_target/release/libai_rebellion.so "$DEST_DIR/"
elif [ -f target/release/libai_rebellion.so ]; then
    cp target/release/libai_rebellion.so "$DEST_DIR/"
fi
echo "✅ libai_rebellion.so compilada exitosamente y copiada a $DEST_DIR"
readelf -l "$DEST_DIR/libai_rebellion.so" | grep -E "LOAD|Align"
readelf -h "$DEST_DIR/libai_rebellion.so" | grep "Type:"

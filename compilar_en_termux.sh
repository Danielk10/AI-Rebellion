#!/usr/bin/env bash
# ==============================================================================
# compilar_en_termux.sh: Compila IA Rebellion directamente en Termux (ARM64)
# vía ADB y extrae el binario a app/src/main/jniLibs/arm64-v8a/
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEVICE="localhost:5555"

echo "🔍 Comprobando conexión ADB con el dispositivo..."
if ! adb -s "$DEVICE" get-state &>/dev/null; then
    echo "⚠️ Dispositivo no detectado en $DEVICE. Conectando..."
    adb-phone connect
fi

TERMUX_HOME="/data/data/com.termux/files/home"

echo "🦀 1. Sincronizando código fuente Rust hacia Termux..."
adb -s "$DEVICE" shell "run-as com.termux sh -c 'mkdir -p $TERMUX_HOME/AI-Rebellion/app/src/main/rust'"
tar --exclude='target' -cz -C "$SCRIPT_DIR/app/src/main" rust | adb -s "$DEVICE" shell "run-as com.termux sh -c 'tar -xz -C $TERMUX_HOME/AI-Rebellion/app/src/main'"


echo "⚙️ 2. Compilando en Termux con 4 núcleos nativos Cortex-A53..."
adb -s "$DEVICE" shell "run-as com.termux sh -c '
    export PREFIX=/data/data/com.termux/files/usr
    export TERMUX_VERSION=0.118.1
    export TMPDIR=/data/data/com.termux/files/usr/tmp
    export PATH=/data/data/com.termux/files/usr/bin:\$PATH
    export HOME=/data/data/com.termux/files/home
    export GOMAXPROCS=4
    export CARGO_BUILD_JOBS=4
    cd /data/data/com.termux/files/home/AI-Rebellion/app/src/main/rust
    cargo build --release
'"

echo "📥 3. Extrayendo libai_rebellion.so compilado..."
DEST_DIR="$SCRIPT_DIR/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$DEST_DIR"
adb -s "$DEVICE" exec-out "run-as com.termux cat /data/data/com.termux/files/home/AI-Rebellion/app/src/main/rust/target/release/libai_rebellion.so" > "$DEST_DIR/libai_rebellion.so"


echo "✅ Binario nativo actualizado exitosamente en $DEST_DIR/libai_rebellion.so:"
ls -lh "$DEST_DIR/libai_rebellion.so"
readelf -l "$DEST_DIR/libai_rebellion.so" | grep -E "LOAD|Align" || true
readelf -h "$DEST_DIR/libai_rebellion.so" | grep "Type:" || true

#!/bin/bash

set -eu

trap '[[ -d "$TEMP_DIR" ]] && rm -rf "$TEMP_DIR"' EXIT

check_file() {
    local file=$1
    if [[ ! -f "$file" ]]; then
        echo "Error: Cannot find $file" >&2
        exit 1
    fi
}

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

ROOT_DIR=$(cd "$SCRIPT_DIR" && pwd)

cd "$ROOT_DIR"

echo "Working directory set to: $ROOT_DIR"

MODE="debug"
CARGO_FLAGS=""
TEMP_DIR=$(mktemp -d -t fusion-plugin-XXXXXX)

for arg in "$@"; do
  if [[ "$arg" == "--release" ]]; then
    MODE="release"
    CARGO_FLAGS="--release"
  fi
done

wasm_path="$ROOT_DIR/target/wasm32-wasip2/$MODE/rust.wasm"

echo "Mode: $MODE"

if [[ ! -d "$TEMP_DIR" ]]; then
  echo "Cannot create temporary directory"
  exit 1
fi

check_file "manifest.toml"
cp manifest.toml "$TEMP_DIR/manifest.toml"

if [[ -d "config" ]]; then
    cp -r "config" "$TEMP_DIR/"
fi

echo "Building WASM module..."

# Патч для автономности проекта, если он внутри чужого воркспейса
if ! grep -q "\[workspace\]" Cargo.toml; then
    printf "\n[workspace]\n" >> Cargo.toml
fi

cargo build --target wasm32-wasip2 $CARGO_FLAGS

check_file "$wasm_path"
cp "$wasm_path" "$TEMP_DIR/module.wasm"

echo "Creating plugin bundle..."
(
    cd "$TEMP_DIR"
    zip -qr "$ROOT_DIR/target/rust.fus" .
)

echo "Success! Output: ./target/rust.fus"

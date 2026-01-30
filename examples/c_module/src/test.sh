#!/bin/bash

#clang
wit-bindgen c ../../../crates/module-engine/wit
/home/irisu/wasi-sdk/wasi-sdk-27.0-x86_64-linux/bin/clang module.c plugin.c plugin_component_type.o -o my-core.wasm -mexec-model=reactor
wasm-tools component new ./my-core.wasm -o ../build/module.wasm
cd ../build
zip example.lym *
mv example.lym ~/.config/nethalym/modules/

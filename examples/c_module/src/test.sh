#!/bin/bash

#clang
cd generated
wit-bindgen c ../wit-plugin
wit-bindgen c ../wit-fusion
cd ..
/home/irisu/wasi-sdk/wasi-sdk-27.0-x86_64-linux/bin/clang ./generated/*.c ./generated/*.o module.c -O3 -o my-core.wasm -mexec-model=reactor
wasm-tools component new ./my-core.wasm -o ../build/module.wasm
cd ../build
zip example.lym *
mv example.lym ~/.config/nethalym/modules/

#!/bin/bash

clang --target=wasm32 -O3 -nostdlib -std=c11 -Wl,--no-entry -Wl,--export-all -I../include ./src/module.c ./src/allocator/o1heap.c ./src/printf/printf.c ./src/printf/print.c -o build/module.wasm
cd build
zip example.lym *
mv example.lym ~/.config/nethalym/modules/

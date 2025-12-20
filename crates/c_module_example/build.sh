#!/bin/bash

clang --target=wasm32 -O3 -nostdlib -std=c11 -Wl,--no-entry -Wl,--export-all ./module.c ./allocator/o1heap.c ./printf/printf.c -o module/module.wasm
cd module
zip example.lym *
mv example.lym ~/.config/nethalym/modules/

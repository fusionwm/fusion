#pragma once

#include "../types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef u32 Window;
typedef void * Windows;

/* IMPORT */
__attribute__((import_module("env")))
__attribute__((import_name("set_window_size")))
void set_window_size(Window window, u32 width, u32 height);

__attribute__((import_module("env")))
__attribute__((import_name("set_window_pos")))
void set_window_pos(Window window, u32 x, u32 y);

__attribute__((import_module("env")))
__attribute__((import_name("get_elements")))
Windows get_elements();

__attribute__((export_name("rearrange_windows")))
void rearrange_windows();

#ifdef __cplusplus
}
#endif

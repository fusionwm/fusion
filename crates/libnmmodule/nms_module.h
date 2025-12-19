#pragma once
#include "nms_defs.h"

#ifdef __cplusplus
extern "C" {
#endif

__attribute__((export_name("module_init")))
void module_init();

__attribute__((export_name("module_tick")))
void module_tick();

__attribute__((export_name("module_stop")))
void module_stop();

__attribute__((export_name("module_heap_allocated")))
usize module_heap_allocated();

__attribute__((export_name("module_heap_capacity")))
usize module_heap_capacity();

__attribute__((export_name("module_heap_free")))
usize module_heap_free();

__attribute__((export_name("module_alloc")))
void *module_alloc(usize size);

__attribute__((export_name("module_free")))
void module_free(void *ptr);

__attribute__((export_name("module_on_failure")))
void *module_on_failure();

__attribute__((export_name("module_restore")))
void module_restore(void *state);

#ifdef __cplusplus
}
#endif

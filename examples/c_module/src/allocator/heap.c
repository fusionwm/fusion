#pragma once

#include <fusion/types.h>
#include <fusion/module.h>
#include "o1heap.h"

O1HeapInstance *heap_instance;
extern u8 __heap_base;

void *module_alloc(usize len) {
    return o1heapAllocate(heap_instance, len);
}

void module_free(void *ptr) {
    o1heapFree(heap_instance, ptr);
}

usize module_heap_allocated() {
    return o1heapAllocated(heap_instance);
}

usize module_heap_free() {
    return module_heap_capacity() - module_heap_allocated();
}

usize module_heap_capacity() {
    return o1heapCapacity(heap_instance);
}

#include "plugin.h"

#define debug fusion_compositor_general_debug
#define info fusion_compositor_general_info
#define warn fusion_compositor_general_warn
#define error fusion_compositor_general_error

void init();
void tick();
void stop();

void exports_plugin_init() {
    init();
}

void exports_plugin_tick() {
    tick();
}

void exports_plugin_stop() {
    stop();
}

uint64_t exports_plugin_heap_allocated() {
    return 0;
}

uint64_t exports_plugin_heap_capacity() {
    return 0;
}

uint64_t exports_plugin_heap_free() {
    return 0;
}

void init() {
    plugin_string_t str;
    plugin_string_set(&str, "Hello, world!");

    info(&str);
}

void tick() {}
void stop() {}

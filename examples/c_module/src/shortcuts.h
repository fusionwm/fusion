#include "generated/general.h"
#include "generated/compositor.h"

#define debug plugin_general_logging_debug
#define info  plugin_general_logging_info
#define warn  plugin_general_logging_warn
#define error plugin_general_logging_error

//config-get: func(path: string) -> string;
//config-delete: func(path: string);
#define config_get plugin_general_config_config_get
#define config_delete plugin_general_config_config_delete

typedef general_string_t string_t;
#define set_string general_string_set

typedef fusion_compositor_window_manager_list_window_id_t list_window;
#define get_elements fusion_compositor_window_manager_get_elements

#define get_output_size fusion_compositor_window_manager_get_output_size

void init(void);
void tick(void);
void stop(void);
void rearrange_windows(void);

void exports_general_init(void) {
    init();
}

void exports_compositor_tick(void) {
    tick();
}

void exports_compositor_stop(void) {
    stop();
}

uint64_t exports_compositor_heap_allocated(void) {
    return 0;
}

uint64_t exports_compositor_heap_capacity(void) {
    return 0;
}

uint64_t exports_compositor_heap_free(void) {
    return 0;
}

void exports_compositor_rearrange_windows(void) {
    rearrange_windows();
}

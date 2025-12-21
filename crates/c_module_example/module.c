#include "../libnthm/ui.h"
#include "allocator/o1heap.h"
#include "heap.c"
#include "print.c"
#include <limits.h>

void module_init() {
    info("Module init");
    heap_instance = o1heapInit((void*)&__heap_base, INT_MAX);

    struct DesktopOptions options = {
        .title = "C Module Example",
        .resizable = true,
        .decorations = true,
    };

    struct WindowLayer layer = {
        .type = LAYER_DESKTOP,
        .options = options,
    };

    Window window = create_window("example", &layer, 800, 600);
    //destroy_window(window);
}

void module_tick() {
    info("Module tick");
}

void module_stop() {
    info("Module stopped");
}

#include "../libnmmodule/nms_net_socket_udp.h"
#include "allocator/o1heap.h"
#include "heap.c"
#include "print.c"
#include <limits.h>

void module_init() {
    heap_instance = o1heapInit((void*)&__heap_base, INT_MAX);

    i32 id = nms_net_socket_udp_create("127.0.0.1:12345");
    if (id < 0) {
        info("Failed to create socket: %d", id);
        return;
    }

    info("Socket created");
    i32 result = nms_net_socket_udp_connect(id, "127.0.0.1:12346");
    if (result < 0) {
        info("Failed to connect to server");
        return;
    }

    nms_net_socket_udp_send(id, "Hello, World!", 14);
    char buffer[1024];
    nms_net_socket_udp_recv(id, buffer, 1024);
    info("Received message: %s", buffer);
    nms_net_socket_udp_shutdown(id);
}

void module_tick() {}

void module_stop() {
    info("Module stopped");
}

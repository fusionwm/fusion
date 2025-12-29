#pragma once

#include "types.h"

#ifdef __cplusplus
extern "C" {
#endif

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_udp_create")))
i32 nms_net_socket_udp_create(const char* bind_addr);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_udp_connect")))
i32 nms_net_socket_udp_connect(i32 id, const char* remote_addr);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_udp_send")))
i64 nms_net_socket_udp_send(i32 id, void *data, i64 length);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_udp_recv")))
i64 nms_net_socket_udp_recv(i32 id, void *buffer, i64 length);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_udp_shutdown")))
i32 nms_net_socket_udp_shutdown(i32 id);

#ifdef __cplusplus
}
#endif

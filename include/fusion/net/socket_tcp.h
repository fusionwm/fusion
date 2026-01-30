#pragma once

#include "../types.h"

#ifdef __cplusplus
extern "C" {
#endif

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_tcp_create")))
i32 nms_net_socket_tcp_create(void);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_tcp_connect")))
i32 nms_net_socket_tcp_connect(i32 id, const char* addr);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_tcp_send")))
i32 nms_net_socket_tcp_send(i32 id, void *data, usize length);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_tcp_recv")))
i64 nms_net_socket_tcp_recv(i32 id, void *buffer, usize length);

__attribute__((import_module("env")))
__attribute__((import_name("nms_net_socket_tcp_shutdown")))
i32 nms_net_socket_tcp_shutdown(i32 id);

#ifdef __cplusplus
}
#endif

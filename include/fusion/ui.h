#pragma once

#include "types.h"
#include "ui/types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum TargetMonitorType {
    TARGET_MONITOR_TYPE_PRIMARY,
    TARGET_MONITOR_TYPE_NAME,
    TARGET_MONITOR_TYPE_INDEX,
    TARGET_MONITOR_TYPE_ALL,
} TargetMonitorType;

typedef struct TargetMonitor {
    TargetMonitorType type;
    union {
        const char* name;
        i32 index;
    } data;
} TargetMonitor;

typedef enum Anchor {
    ANCHOR_TOP,
    ANCHOR_BOTTOM,
    ANCHOR_LEFT,
    ANCHOR_RIGHT,
} Anchor;

typedef enum WindowLayerType {
    LAYER_DESKTOP,
    LAYER_TOP,
    LAYER_BOTTOM,
    LAYER_OVERLAY,
    LAYER_BACKGROUND,
} WindowLayerType;

struct DesktopOptions {
    const char* title;
    u32 __padding;
    bool resizable;
    bool decorations;
};

struct SpecialOptions {
    Anchor anchor;
    u32 exclusive_zone;
    TargetMonitor target;
};

struct WindowLayer {
    WindowLayerType type;
    u32 __padding;
    union {
        struct DesktopOptions desktop;
        struct SpecialOptions special;
    } options;
};

__attribute__((import_module("env")))
__attribute__((import_name("create_window")))
Window create_window(const char* id, struct WindowLayer *layer, i32 width, i32 height);

__attribute__((import_module("env")))
__attribute__((import_name("destroy_window")))
void destroy_window(Window window);

__attribute__((import_module("env")))
__attribute__((import_name("set_window_title")))
void set_window_title(Window window, const char* title);

__attribute__((import_module("env")))
__attribute__((import_name("resize_window")))
void resize_window(Window window, i32 width, i32 height);

__attribute__((import_module("env")))
__attribute__((import_name("move_window")))
void move_window(Window window, i32 x, i32 y);

__attribute__((import_module("env")))
__attribute__((import_name("set_window_visibility")))
void set_window_visibility(Window window, bool visible);

__attribute__((import_module("env")))
__attribute__((import_name("push_draw_command")))
void push_draw_command(Window window, CommandPool pool);

#ifdef __cplusplus
}
#endif

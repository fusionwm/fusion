#pragma once

#include "../types.h"
#include "../../import.h"

#ifdef __cplusplus
extern "C" {
#endif

struct Label {
    Color color;
    u32 width;
    u32 height;
};

static void layout(Label *label) {
    nms_log_info("Label logic");
}

static void update(Label *label) {
    nms_log_info("Label update");
}

static void draw(Label *label, CommandPool *pool) {
    nms_log_info("Label draw");
}

// Правильный тип для таблицы функций
typedef void (*LabelLayoutFunc)(Label*);
typedef void (*LabelUpdateFunc)(Label*);
typedef void (*LabelDrawFunc)(Label*, CommandPool*);

// Объединенный тип для хранения всех функций
typedef union {
    LabelLayoutFunc layout;
    LabelUpdateFunc update;
    LabelDrawFunc draw;
    void* ptr; // Для общего хранения
} LabelFunction;

// 3. Таблица функций КОНКРЕТНОГО типа виджета
static LabelFunction label_functions[] = {
    { .layout = layout },    // INDEX 0: layout
    { .update = update },    // INDEX 1: update
    { .draw = draw },        // INDEX 2: draw
};

#ifdef __cplusplus
}
#endif

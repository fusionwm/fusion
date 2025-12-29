#pragma once

#include "../types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    FUNC_UPDATE_LAYOUT = 0,
    FUNC_UPDATE_LOGIC,
    FUNC_COLLECT_RENDER,
    FUNC_HANDLE_EVENT,
    FUNC_DESTROY,
    FUNC_COUNT_PER_TYPE
} WidgetFunction;

#define TABLE_INDEX(type, func) \
    ((type) * FUNC_COUNT_PER_TYPE + (func))

#ifdef __cplusplus
}
#endif

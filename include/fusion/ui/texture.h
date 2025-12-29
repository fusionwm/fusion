
#pragma once

#include "../types.h"
#include "types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    usize id;
} TextureHandle;

typedef struct {
    usize id;
    u32 width;
    u32 height;
} SvgHandle;

typedef enum {
    HANDLE_TEXTURE,
    HANDLE_SVG
} HandleTag;

typedef struct {
    HandleTag tag;
    union {
        TextureHandle texture;
        SvgHandle svg;
    } data;
} Handle;

struct Texture {
  Color color;
  Handle handle;
};

#ifdef __cplusplus
}
#endif

#pragma once

#include "../types.h"
#include "texture.h"
#include "types.h"

#ifdef __cplusplus
extern "C" {
#endif

struct DrawRectCommand {
    Bounds rect;
    Color color;
    Stroke stroke;
};

struct DrawTextureCommand {
    Bounds rect;
    Texture texture;
    Stroke stroke;
};

struct DrawTextCommand {
    u32 size;
    Color color;
    Vec2 position;
    Font *font;
};

typedef enum DrawCommandType {
    RECT,
    TEXTURE,
    TEXT
} DrawCommandType;

struct DrawCommand {
    DrawCommandType type;
    union {
        struct DrawRectCommand rect;
        struct DrawTextureCommand texture;
        struct DrawTextCommand text;
    } data;
};

#ifdef __cplusplus
}
#endif

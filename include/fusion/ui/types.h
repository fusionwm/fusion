#pragma once

#include "../types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef long long Window;
typedef long long CommandPool;
typedef long long Font;

Struct(Color,
    u8 r;
    u8 g;
    u8 b;
    u8 a;
);

Struct(Vec2,
    f32 x;
    f32 y;
);

Struct(Bounds,
    Vec2 position;
    Vec2 size;
);

Struct(Stroke, /// Left, Right, Top, Bottom
    Color color[4];
    f32 width;
);

#ifdef __cplusplus
}
#endif

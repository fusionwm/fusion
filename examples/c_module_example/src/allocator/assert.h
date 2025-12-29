#pragma once

#ifndef NDEBUG
// Реализация assert для WASM без stdlib
#define assert(expr) \
    do { \
        if (!(expr)) { \
            /* Вывод сообщения об ошибке */ \
            const char* msg = "Assertion failed: " #expr "\n"; \
            __builtin_trap(); \
        } \
    } while(0)
#else
#define assert(expr) ((void)0)
#endif

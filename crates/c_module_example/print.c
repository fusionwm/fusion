#pragma once

#include "printf/printf.h"
#include "../libnthm/import.h"

static char message[1024] = { 0 };
static usize message_pointer = 0;

// Кастомный _putchar для буферизации
void _putchar(char character) {
    if (message_pointer < sizeof(message) - 1) {
        message[message_pointer++] = character;
    }

    // Если буфер полный или символ новой строки - флашим
    if (character == '\0' || message_pointer >= sizeof(message) - 1) {
        nms_log_info(message);
        message_pointer = 0;
    }
}

// Обертка для info()
void info(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);

    // Сбрасываем позицию буфера
    message_pointer = 0;

    // Форматируем в буфер через printf
    vprintf(fmt, args);
    _putchar('\0');

    va_end(args);
}

#pragma once

#include "nms_defs.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Array {
    void *ptr;
    usize len;
} Array;

typedef enum ValueType {
    VALUE_INTEGER,
    VALUE_UNSIGNED_INTEGER,
    VALUE_FLOAT,
    VALUE_BOOLEAN,
    VALUE_ENUM,
    VALUE_LOCALIZATION_KEY,
    VALUE_STRING,
    VALUE_ARRAY
} ValueType;

typedef union ValueData {
    i32 integer;
    u32 unsigned_integer;
    f32 float_value;
    bool boolean;
    const char* enumeration;
    const char* localization_key;
    const char* string;
    Array array;
} ValueData;

typedef struct Value {
    ValueType type;
    u32 _padding;
    ValueData data;
} Value;

// =========== ЛОГИРОВАНИЕ ===========
__attribute__((import_module("env")))
__attribute__((import_name("nms_log_info")))
void nms_log_info(const char* message);

__attribute__((import_module("env")))
__attribute__((import_name("nms_log_warn")))
void nms_log_warn(const char* message);

__attribute__((import_module("env")))
__attribute__((import_name("nms_log_error")))
void nms_log_error(const char* message);

__attribute__((import_module("env")))
__attribute__((import_name("nms_config_get")))
///Требуется освобождение памяти
const Value *nms_config_get(const char* key);

__attribute__((import_module("env")))
__attribute__((import_name("nms_config_delete")))
void nms_config_delete(const char* key);

#ifdef __cplusplus
}
#endif

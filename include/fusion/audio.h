#pragma once

#include "types.h"

#ifdef __cplusplus
extern "C" {
#endif

__attribute__((import_module("env")))
__attribute__((import_name("nms_audio_set_volume")))
void nms_audio_set_volume(i32 volume);

__attribute__((import_module("env")))
__attribute__((import_name("nms_audio_mute")))
void nms_audio_mute(i32 enable);

#ifdef __cplusplus
}
#endif

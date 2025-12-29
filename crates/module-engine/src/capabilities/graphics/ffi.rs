use std::{
    ffi::{c_char, c_int},
    hint::unreachable_unchecked,
};

use graphics::reexports::{DesktopOptions, SpecialOptions, TargetMonitor, WindowLayer};
use wasmtime::Caller;

use crate::{capabilities::read_wasm_string, context::ExecutionContext};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CTargetMonitorType {
    Primary = 0,
    Name = 1,
    Index = 2,
    All = 3,
}

#[repr(C)]
#[derive(Copy, Clone)]
union CTargetMonitorData {
    pub name: *const c_char,
    pub index: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CTargetMonitor {
    pub monitor_type: CTargetMonitorType,
    pub data: CTargetMonitorData,
}

fn convert_target_monitor_fast(target: &CTargetMonitor) -> TargetMonitor {
    unsafe {
        match target.monitor_type {
            CTargetMonitorType::Primary => TargetMonitor::Primary,
            CTargetMonitorType::All => TargetMonitor::All,
            CTargetMonitorType::Index => {
                // Прямое чтение числа из union (очень быстро)
                TargetMonitor::Index(target.data.index as usize)
            }
            CTargetMonitorType::Name => {
                // Самая «дорогая» операция из-за аллокации строки
                let c_str = std::ffi::CStr::from_ptr(target.data.name);
                TargetMonitor::Name(c_str.to_string_lossy().into_owned())
            }
        }
    }
}

//#[repr(C)]
#[repr(C)]
pub struct CWindowLayer {
    type_: c_int,
    options: CWindowLayerOptions,
}

#[repr(C)]
union CWindowLayerOptions {
    pub desktop: CDesktopOptions,
    pub special: CSpecialOptions,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CDesktopOptions {
    pub title: *const c_char,
    pub resizable: bool,
    pub decorations: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CSpecialOptions {
    pub anchor: graphics::reexports::Anchor,
    pub exclusive_zone: c_int,
    pub target: CTargetMonitor,
}

/// Самая быстрая конвертация
pub fn convert_window_layer_fast(
    c_layer: &CWindowLayer,
    caller: &mut Caller<'_, ExecutionContext>,
) -> WindowLayer {
    unsafe {
        match c_layer.type_ {
            0 => {
                let d = c_layer.options.desktop;
                WindowLayer::Desktop(DesktopOptions {
                    title: if d.title.is_null() {
                        String::new()
                    } else {
                        read_wasm_string(caller, d.title as i32).to_string()
                    },
                    resizable: d.resizable,
                    decorations: d.decorations,
                })
            }
            t @ 1..=4 => {
                let s = c_layer.options.special;
                let opt = SpecialOptions {
                    anchor: s.anchor,
                    exclusive_zone: s.exclusive_zone as u32,
                    target: convert_target_monitor_fast(&s.target),
                };
                match t {
                    1 => WindowLayer::Top(opt),
                    2 => WindowLayer::Bottom(opt),
                    3 => WindowLayer::Overlay(opt),
                    _ => WindowLayer::Background(opt),
                }
            }
            _ => unreachable_unchecked(), // Подсказка компилятору, что других вариантов нет
        }
    }
}

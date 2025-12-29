use std::ffi::c_char;

use bytemuck::NoUninit;
use wasmtime::{AsContextMut, Func, Memory, Val};

use crate::module::config::Value;

#[repr(C)]
#[derive(Copy, Clone)]
enum FfiValueType {
    Integer,
    UnsignedInteger,
    Float,
    Boolean,
    Enum,
    LocalizationKey,
    String,
    Array,
}

#[repr(C)]
#[derive(Copy, Clone)]
union FfiValueData {
    integer: i32,
    unsigned_integer: u32,
    float_value: f32,
    boolean: i32,
    enumeration: *const c_char,
    localization_key: *const c_char,
    string: *const c_char,
    array: FfiArray,
}

unsafe impl bytemuck::Zeroable for FfiValueData {}
unsafe impl bytemuck::Pod for FfiValueData {}

#[repr(C)]
#[derive(Copy, Clone)]
struct FfiArray {
    ptr: i32,
    len: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiValue {
    kind: FfiValueType,
    data: FfiValueData,
}

unsafe impl bytemuck::Zeroable for FfiValue {}
unsafe impl bytemuck::Pod for FfiValue {}

pub struct ModuleAllocator<T: AsContextMut> {
    alloc: Func,
    store: T,
    memory: Memory,
}

impl<T: AsContextMut> ModuleAllocator<T> {
    pub fn new(alloc: Func, store: T, memory: Memory) -> Self {
        ModuleAllocator {
            alloc,
            store,
            memory,
        }
    }

    pub fn alloc(&mut self, len: usize) -> (*const u32, &mut [u8]) {
        let mut results = [Val::I32(0); 1];
        self.alloc
            .call(&mut self.store, &[Val::I64(len as i64)], &mut results)
            .unwrap();
        let ptr = results[0].unwrap_i32();

        (
            ptr as *const u32,
            &mut self.memory.data_mut(&mut self.store)[ptr as usize..(ptr as usize + len)],
        )
    }

    pub fn alloc_string(&mut self, value: &str) -> *const c_char {
        let bytes = value.as_bytes();
        let len = bytes.len() + 1; // +1 для '\0'
        let (ptr, data) = self.alloc(len);
        data[..bytes.len()].copy_from_slice(bytes);
        data[bytes.len()] = 0;
        ptr as *const c_char
    }

    pub fn alloc_bytes<D: NoUninit>(&mut self, data: &D) -> *const u32 {
        let data = bytemuck::bytes_of(data);
        let (ptr, module_memory) = self.alloc(data.len());
        module_memory.copy_from_slice(data);
        ptr
    }
}

impl FfiValue {
    pub fn new<T: AsContextMut>(value: Value, allocator: &mut ModuleAllocator<T>) -> Self {
        let (kind, data) = match value {
            Value::Integer(value) => (FfiValueType::Integer, FfiValueData { integer: value }),
            Value::UnsignedInteger(value) => (
                FfiValueType::UnsignedInteger,
                FfiValueData {
                    unsigned_integer: value,
                },
            ),
            Value::Float(value) => (FfiValueType::Float, FfiValueData { float_value: value }),
            Value::Boolean(value) => (
                FfiValueType::Boolean,
                FfiValueData {
                    boolean: value.into(),
                },
            ),
            Value::Enumeration(value) => (
                FfiValueType::Enum,
                FfiValueData {
                    enumeration: allocator.alloc_string(&value.0),
                },
            ),
            Value::LocalizationKey(value) => (
                FfiValueType::LocalizationKey,
                FfiValueData {
                    localization_key: allocator.alloc_string(&value.0),
                },
            ),
            Value::String(value) => (
                FfiValueType::String,
                FfiValueData {
                    string: allocator.alloc_string(&value.0),
                },
            ),
            Value::Array(value) => {
                let len = value.0.len();

                //TODO fix
                let mut buffer = vec![0; len * size_of::<FfiValueData>()];

                let mut offset = 0;
                value.0.into_iter().for_each(|item| {
                    let value = FfiValue::new(item, allocator);
                    let bytes = bytemuck::bytes_of(&value.data);
                    buffer[offset..offset + bytes.len()].copy_from_slice(bytes);
                    offset += bytes.len();
                });

                let (ptr, memory) = allocator.alloc(size_of::<FfiValueData>() * len);
                memory.copy_from_slice(&buffer);

                let array = FfiArray {
                    ptr: ptr as i32,
                    len: len as i32,
                };

                (FfiValueType::Array, FfiValueData { array })
            }
        };
        FfiValue { kind, data }
    }
}

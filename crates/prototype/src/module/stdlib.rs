use wasmtime::{AsContextMut, Instance, Memory, Store, TypedFunc};

use crate::module::context::ExecutionContext;

#[allow(dead_code)]
pub struct StandardLibrary {
    init: TypedFunc<(), ()>,
    tick: TypedFunc<(), ()>,
    stop: TypedFunc<(), ()>,

    on_failure: Option<TypedFunc<(), i32>>,
    restore: Option<TypedFunc<i32, ()>>,

    heap_allocated: TypedFunc<(), i64>,
    heap_capacity: TypedFunc<(), i64>,
    heap_free: TypedFunc<(), i64>,
}

impl StandardLibrary {
    pub fn new(instance: &Instance, mut store: impl AsContextMut) -> Result<Self, crate::Error> {
        let init = instance.get_typed_func::<(), ()>(&mut store, "module_init")?;
        let tick = instance.get_typed_func::<(), ()>(&mut store, "module_tick")?;
        let stop = instance.get_typed_func::<(), ()>(&mut store, "module_stop")?;

        let on_failure = instance
            .get_typed_func::<(), i32>(&mut store, "module_on_failure")
            .ok();

        let restore = if on_failure.is_some() {
            instance
                .get_typed_func::<i32, ()>(&mut store, "module_restore")
                .ok()
        } else {
            None
        };

        let heap_allocated =
            instance.get_typed_func::<(), i64>(&mut store, "module_heap_allocated")?;
        let heap_capacity =
            instance.get_typed_func::<(), i64>(&mut store, "module_heap_capacity")?;
        let heap_free = instance.get_typed_func::<(), i64>(&mut store, "module_heap_free")?;

        Ok(Self {
            init,
            tick,
            stop,

            on_failure,
            restore,

            heap_allocated,
            heap_capacity,
            heap_free,
        })
    }

    pub async fn init(&self, mut store: &mut Store<ExecutionContext>) -> Result<(), crate::Error> {
        self.init.call_async(&mut store, ()).await?;
        Ok(())
    }

    pub async fn tick(&self, mut store: &mut Store<ExecutionContext>) -> Result<(), crate::Error> {
        self.tick.call_async(&mut store, ()).await?;
        Ok(())
    }

    pub async fn stop(&self, mut store: &mut Store<ExecutionContext>) -> Result<(), crate::Error> {
        self.stop.call_async(&mut store, ()).await?;
        Ok(())
    }

    pub fn is_support_restore(&self) -> bool {
        self.restore.is_some()
    }

    pub async fn get_restore_state(
        &self,
        memory: Memory,
        mut store: &mut Store<ExecutionContext>,
    ) -> Result<Vec<u8>, crate::Error> {
        let func = self.on_failure.as_ref().unwrap();
        let ptr = func.call_async(&mut store, ()).await? as usize;

        if ptr == 0 {
            return Ok(Vec::new());
        }

        let memory = memory.data(&mut store);

        let length_bytes = [
            memory[ptr],
            memory[ptr + 1],
            memory[ptr + 2],
            memory[ptr + 3],
        ];

        let length = u32::from_le_bytes(length_bytes);

        if length == 0 {
            return Ok(Vec::new());
        }

        let mut data = bytemuck::bytes_of(&(ptr as u32)).to_vec();
        data.extend_from_slice(&memory[ptr..ptr + length as usize]);
        Ok(data)
    }

    pub async fn restore(
        &self,
        mut store: &mut Store<ExecutionContext>,
        memory: Memory,
        state: Vec<u8>,
    ) -> Result<(), crate::Error> {
        debug_assert!(state.len() >= 4);
        let ptr_bytes = [state[0], state[1], state[2], state[3]];
        let ptr = u32::from_le_bytes(ptr_bytes) as usize;

        let length_bytes = [state[4], state[5], state[6], state[7]];
        let length = u32::from_le_bytes(length_bytes) as usize;

        memory.data_mut(&mut store)[ptr..ptr + length].copy_from_slice(&state[4..]);

        let func = self.restore.as_ref().unwrap();
        func.call_async(&mut store, ptr as i32).await?;

        Ok(())
    }
}

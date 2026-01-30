use wasmtime::component::HasData;

use crate::{
    capabilities::general::fusion::compositor::general, context::ExecutionContext, debug, error,
    info, warn,
};

wasmtime::component::bindgen!("plugin");

impl HasData for ExecutionContext {
    type Data<'a> = &'a mut ExecutionContext;
}

impl general::Host for ExecutionContext {
    #[inline]
    fn debug(&mut self, message: String) {
        debug!(self.logger(), "{message}");
    }

    #[inline]
    fn info(&mut self, message: String) {
        info!(self.logger(), "{message}");
    }

    #[inline]
    fn warn(&mut self, message: String) {
        warn!(self.logger(), "{message}");
    }

    #[inline]
    fn error(&mut self, message: String) {
        error!(self.logger(), "{message}");
    }

    #[inline]
    fn config_get(&mut self, path: String) -> String {
        let config = self.config();
        let _value = config.get_value(&path).unwrap();
        String::new()
    }

    #[inline]
    fn config_delete(&mut self, _key: String) {}
}

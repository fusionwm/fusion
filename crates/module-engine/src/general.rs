use wasmtime::component::HasData;

use crate::{
    context::ExecutionContext,
    engine::InnerContext,
    general::plugin::general::{config, logging},
};

wasmtime::component::bindgen!("general");

impl<I: InnerContext> HasData for ExecutionContext<I> {
    type Data<'a> = &'a mut ExecutionContext<I>;
}

impl<I: InnerContext> logging::Host for ExecutionContext<I> {
    fn debug(&mut self, message: String) {
        log::warn!("{message}");
        crate::debug!(self.logger(), "{message}");
    }

    fn info(&mut self, message: String) {
        log::warn!("{message}");
        crate::info!(self.logger(), "{message}");
    }

    fn warn(&mut self, message: String) {
        log::warn!("{message}");
        crate::warn!(self.logger(), "{message}");
    }

    fn error(&mut self, message: String) {
        log::error!("{message}");
        crate::error!(self.logger(), "{message}");
    }
}

impl<I: InnerContext> config::Host for ExecutionContext<I> {
    fn config_get(&mut self, path: String) -> String {
        let config = self.config();
        let _value = config.get_value(&path).unwrap();
        String::new()
    }

    fn config_delete(&mut self, _key: String) {}
}

use wasmtime::component::HasData;

use crate::{
    context::ExecutionContext,
    engine::InnerContext,
    general::plugin::general::{config, logging},
};

wasmtime::component::bindgen!({
    path: "../../specs/plugin-base",
    world: "general",
});

impl<I: InnerContext> HasData for ExecutionContext<I> {
    type Data<'a> = &'a mut ExecutionContext<I>;
}

impl<I: InnerContext> logging::Host for ExecutionContext<I> {
    fn debug(&mut self, message: String) {
        tracing::debug!("{message}");
    }

    fn info(&mut self, message: String) {
        tracing::info!("{message}");
    }

    fn warn(&mut self, message: String) {
        tracing::warn!("{message}");
    }

    fn error(&mut self, message: String) {
        tracing::error!("{message}");
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

use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::engine::InnerContext;
use crate::{config::Config, logging::Logger};
use std::path::PathBuf;

pub struct ExecutionContext<I: InnerContext> {
    logger: Logger,
    config: Config,
    wasi: WasiCtx,
    table: ResourceTable,
    pub inner: I,
}

impl<I: InnerContext> ExecutionContext<I> {
    pub fn new(config: Config, log: PathBuf, inner: I) -> Self {
        ExecutionContext {
            logger: Logger::new(log),
            config,
            inner,
            wasi: WasiCtx::default(),
            table: ResourceTable::new(),
        }
    }

    pub const fn logger(&mut self) -> &mut Logger {
        &mut self.logger
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }
}

impl<I: InnerContext> WasiView for ExecutionContext<I> {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

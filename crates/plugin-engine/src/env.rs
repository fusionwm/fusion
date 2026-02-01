use wasmtime::component::{Component, Linker};

use crate::{
    context::ExecutionContext,
    engine::{Bindings, InnerContext, PluginID, PluginStatus},
    general::General,
    manifest::Manifest,
    table::CapabilityTable,
};

#[allow(dead_code)]
pub struct PluginEnvironment<I: InnerContext> {
    path: String,
    manifest: Manifest,
    component: Component,
    status: PluginStatus,
    bindings: Bindings<I>,
}

impl<I: InnerContext> PluginEnvironment<I> {
    pub const fn new(
        path: String,
        manifest: Manifest,
        component: Component,
        bindings: Bindings<I>,
    ) -> Self {
        Self {
            path,
            manifest,
            component,
            status: PluginStatus::Running,
            bindings,
        }
    }

    #[must_use]
    pub const fn path(&self) -> &str {
        self.path.as_str()
    }

    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    #[must_use]
    pub const fn component(&self) -> &Component {
        &self.component
    }

    #[must_use]
    pub const fn status(&self) -> PluginStatus {
        self.status
    }

    #[must_use]
    pub fn bindings_mut(&mut self) -> &mut Bindings<I> {
        &mut self.bindings
    }

    pub fn instantiate_general_api(
        &mut self,
        linker: &Linker<ExecutionContext<I>>,
    ) -> Result<General, Box<dyn std::error::Error>> {
        Ok(General::instantiate(
            self.bindings.store_mut(),
            &self.component,
            &linker,
        )?)
    }

    pub fn create_bindings(
        &mut self,
        linker: &mut Linker<ExecutionContext<I>>,
        captable: &mut CapabilityTable<I>,
        plugin_id: PluginID,
    ) {
        let capabilities = unsafe {
            // SAFETY: bindings does not own the capabilities array, so it is safe to dereference it.
            let capabilities = self.manifest().capabilities() as *const [String];
            &*capabilities
        };

        captable.create_bindings(
            capabilities,
            &mut self.bindings,
            &self.component,
            linker,
            plugin_id,
        );
    }
}

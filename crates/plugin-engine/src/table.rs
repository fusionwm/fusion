#![allow(dead_code)]

use crate::{
    engine::{Bindings, PluginID, UntypedPluginBinding},
    general::General,
    wasm::{Component, Linker, Store},
};
use bitflags::bitflags;
use log::info;
use std::collections::{HashMap, HashSet};

use crate::{context::ExecutionContext, engine::InnerContext};

pub trait CapabilityProvider: 'static {
    type Inner: InnerContext;
    fn link_functions(&self, linker: &mut Linker<ExecutionContext<Self::Inner>>);
    fn create_bindings(
        &self,
        store: &mut Store<ExecutionContext<Self::Inner>>,
        component: &Component,
        linker: &Linker<ExecutionContext<Self::Inner>>,
    ) -> Box<dyn UntypedPluginBinding>;
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilityWriteRules: u32 {
        const None = 0;
        const SingleWrite = 1;
        const MultipleWrite = 2;
    }
}

impl CapabilityWriteRules {
    #[inline]
    fn is_multiple_write(self) -> bool {
        self.contains(CapabilityWriteRules::MultipleWrite)
    }

    #[inline]
    fn is_single_write(self) -> bool {
        self.contains(CapabilityWriteRules::SingleWrite)
    }

    fn new_checker(self) -> WriterCounter {
        let max = if self.is_multiple_write() {
            u32::MAX
        } else {
            u32::from(self.is_single_write())
        };

        WriterCounter {
            max,
            current: 0,
            writers: HashSet::new(),
        }
    }
}

pub struct WriterCounter {
    max: u32,
    current: u32,
    writers: HashSet<PluginID>,
}

impl WriterCounter {
    fn add_writer_if_possible(&mut self, plugin_id: PluginID) {
        if self.max == 0 {
            return;
        }
        self.current += 1;
        self.writers.insert(plugin_id);
    }

    fn remove_writer(&mut self, plugin_id: PluginID) {
        if self.max == 0 {
            return;
        }

        if !self.writers.remove(&plugin_id) {
            return;
        }

        self.current -= 1;
    }
}

pub struct Capability<I: InnerContext> {
    checker: WriterCounter,
    rules: CapabilityWriteRules,
    provider: Box<dyn CapabilityProvider<Inner = I>>,
    depends_on: Vec<String>,
}

impl<I: InnerContext> Capability<I> {
    #[inline]
    const fn is_allowed_to_use(&self) -> bool {
        self.checker.max != 0 && self.checker.current < self.checker.max
    }

    #[must_use]
    pub(crate) const fn writers(&self) -> &HashSet<PluginID> {
        &self.checker.writers
    }
}

pub(crate) struct CapabilityTable<I: InnerContext> {
    inner: HashMap<String, Capability<I>>,
}

impl<I: InnerContext> Default for CapabilityTable<I> {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl<I: InnerContext> CapabilityTable<I> {
    pub fn register_capability(
        &mut self,
        name: String,
        rules: CapabilityWriteRules,
        provider: impl CapabilityProvider<Inner = I>,
    ) -> bool {
        if let Some(_get) = self.inner.get(&name) {
            return false;
        }

        info!("Register capability: {name}");
        let checker = rules.new_checker();

        self.inner.insert(
            name,
            Capability {
                checker,
                rules,
                provider: Box::new(provider),
                depends_on: Vec::new(),
            },
        );

        true
    }

    pub fn link(
        &self,
        capabilities: &[String],
        linker: &mut Linker<ExecutionContext<I>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        General::add_to_linker::<_, ExecutionContext<I>>(linker, |store| store)?;
        for requested in capabilities {
            if let Some(capability) = self.inner.get(requested) {
                if capability.is_allowed_to_use() {
                    capability.provider.link_functions(linker);
                } else {
                    panic!("Capability {requested} is not allowed to use");
                }
            } else {
                //TODO
                let error = format!("[TODO] Requested capability {requested} is missing");
                panic!("{error}");
            }
        }

        Ok(())
    }

    pub fn remove_observing(&mut self, capabilities: &[String], plugin_id: PluginID) {
        for requested in capabilities {
            // SAFETY: We have already checked that the capability exists
            let capability = unsafe { self.inner.get_mut(requested).unwrap_unchecked() };
            capability.checker.remove_writer(plugin_id);
        }
    }

    pub fn create_bindings(
        &mut self,
        capabilities: &[String],
        bindings: &mut Bindings<I>,
        component: &Component,
        linker: &mut Linker<ExecutionContext<I>>,
        plugin_id: PluginID,
    ) {
        for requested in capabilities {
            // SAFETY: We have already checked that the capability exists
            let capability = unsafe { self.inner.get_mut(requested).unwrap_unchecked() };
            capability.checker.add_writer_if_possible(plugin_id);
            let binding =
                capability
                    .provider
                    .create_bindings(bindings.store_mut(), component, linker);

            bindings.add(binding);
        }
    }

    pub fn get_capability_by_name(&self, name: &str) -> &Capability<I> {
        self.inner
            .get(name)
            .unwrap_or_else(|| panic!("Capability {name} not found"))
    }
}

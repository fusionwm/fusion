#![allow(dead_code)]

use bitflags::bitflags;
use log::info;
use std::collections::HashMap;
use wasmtime::Func;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilityKind: u32 {
        const Read = 0;
        const SingleWrite = 1;
        const MultipleWrite = 2;
        const All = Self::Read.bits() | Self::SingleWrite.bits() | Self::MultipleWrite.bits();
    }
}

pub struct Capability {
    kind: CapabilityKind,
    functions: Vec<Func>,
}

#[derive(Default)]
pub struct CapabilityTable {
    inner: HashMap<String, Capability>,
}

impl CapabilityTable {
    pub fn register_capability(
        &mut self,
        name: String,
        kind: CapabilityKind,
        functions: Vec<Func>,
    ) -> bool {
        if let Some(_get) = self.inner.get(&name) {
            return false;
        }

        info!("Register capability: {name}");

        self.inner.insert(name, Capability { kind, functions });

        true
    }
}

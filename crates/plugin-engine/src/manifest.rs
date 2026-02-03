#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;

use crate::PluginID;

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    name: String,
    email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleError {
    name: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ParameterValue {
    String,
    Number,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Parameter {
    name: String,
    value: ParameterValue,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigSchema {
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    id: PluginID,
    name: String,
    version: String,
    description: String,
    repository: String,
    authors: Vec<Author>,
    capabilities: Option<Vec<String>>,
    custom: Option<Vec<String>>,
    errors: Option<HashMap<usize, ModuleError>>,
    schema: Option<ConfigSchema>,
}

impl Manifest {
    #[must_use]
    pub const fn id(&self) -> &PluginID {
        &self.id
    }

    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn custom_capabilities(&self) -> Option<&[String]> {
        self.custom.as_deref()
    }

    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        self.capabilities.as_deref().unwrap_or_default()
    }
}

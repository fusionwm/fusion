use bincode::{Decode, Encode};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Encode, Decode)]
pub struct Author {
    name: String,
    email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Encode, Decode)]
pub struct ModuleError {
    name: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize, Encode, Decode)]
pub enum ParameterValue {
    String,
    Number,
}

#[derive(Debug, Clone, Deserialize, Encode, Decode)]
pub struct Parameter {
    name: String,
    value: ParameterValue,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Deserialize, Encode, Decode)]
pub struct ConfigSchema {
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Deserialize, Encode, Decode)]
pub struct Manifest {
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
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn custom_capabilities(&self) -> Option<&[String]> {
        self.custom.as_deref()
    }

    pub fn capabilities(&self) -> &[String] {
        self.capabilities.as_deref().unwrap_or_default()
    }
}

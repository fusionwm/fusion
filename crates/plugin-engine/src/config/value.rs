use std::str::FromStr;

use regex::Regex;

#[derive(Debug, Clone)]
pub struct LocalizationKey(pub String);

impl FromStr for LocalizationKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('$') {
            return Err(());
        }

        Ok(LocalizationKey(s[1..].to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct Enum(pub String);

impl FromStr for Enum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$")
            .unwrap()
            .is_match(s)
            .then_some(Enum(s.to_string()))
            .ok_or(())
    }
}

#[derive(Debug, Clone)]
pub struct Array(pub Vec<Value>);

impl FromStr for Array {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Array(
            s.split(',')
                .map(|s| Value::from_str(s.trim()).unwrap())
                .collect(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Str(pub String);

impl FromStr for Str {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('"') && s.ends_with('"') {
            Ok(Str(s[1..s.len() - 1].to_string()))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    UnsignedInteger(u32),
    Float(f32),
    Boolean(bool),
    Enumeration(Enum),
    LocalizationKey(LocalizationKey),
    String(Str),
    Array(Array),
}

impl FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = if let Ok(value) = s.parse::<i32>() {
            Value::Integer(value)
        } else if let Ok(value) = s.parse::<u32>() {
            Value::UnsignedInteger(value)
        } else if let Ok(value) = s.parse::<f32>() {
            Value::Float(value)
        } else if let Ok(value) = s.parse::<bool>() {
            Value::Boolean(value)
        } else if let Ok(value) = s.parse::<LocalizationKey>() {
            Value::LocalizationKey(value)
        } else if let Ok(value) = s.parse::<Enum>() {
            Value::Enumeration(value)
        } else if let Ok(value) = s.parse::<Str>() {
            Value::String(value)
        } else if let Ok(value) = s.parse::<Array>() {
            Value::Array(value)
        } else {
            panic!("Invalid value")
        };

        Ok(value)
    }
}

impl Value {
    #[must_use]
    pub fn as_enum(&self) -> String {
        match self {
            Value::Enumeration(_) => "enum".to_string(),
            _ => panic!("Value is not an enumeration"),
        }
    }

    #[must_use]
    pub fn as_array_of_enum(&self) -> Vec<String> {
        match self {
            Value::Array(array) => array.0.iter().map(Value::as_enum).collect(),
            _ => panic!("Value is not an array"),
        }
    }
}

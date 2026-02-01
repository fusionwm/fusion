#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ValueType {
    String,
    UInt,
    Int,
    Float,
    Enum,
}

impl ValueType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "String" => ValueType::String,
            "UInt" => ValueType::UInt,
            "Int" => ValueType::Int,
            "Float" => ValueType::Float,
            "Enum" => ValueType::Enum,
            _ => panic!("Invalid value type: {s}"),
        }
    }
}

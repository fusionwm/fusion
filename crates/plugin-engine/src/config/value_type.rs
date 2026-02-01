use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ValueType {
    String,
    UInt,
    Int,
    Float,
    Enum,
}

impl FromStr for ValueType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "String" => ValueType::String,
            "UInt" => ValueType::UInt,
            "Int" => ValueType::Int,
            "Float" => ValueType::Float,
            "Enum" => ValueType::Enum,
            _ => panic!("Invalid value type: {s}"),
        })
    }
}

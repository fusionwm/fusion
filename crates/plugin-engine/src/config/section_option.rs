use std::collections::HashMap;

use tree_sitter::Node;

use crate::config::{read_meta_info, read_meta_name, value::Value, value_type::ValueType};

#[derive(Debug, Clone)]
pub struct SectionOption {
    name: String,
    type_: ValueType,

    default: Option<Value>,
    //Only if the type_ field is ValueType::Enum
    enum_values: Option<Vec<String>>,
}

impl SectionOption {
    pub fn from_node(node: &Node, source: &str) -> Self {
        let name = node
            .child_by_field_name("name")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap()
            .to_string();

        let type_ = ValueType::from_str(
            node.child_by_field_name("type")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap(),
        );

        let mut meta_infos: HashMap<String, Value> = node
            .children(&mut node.walk())
            .filter(|child| child.kind() == "meta_field")
            .map(|child| {
                let name = read_meta_name(&child, source).to_string();
                let value = read_meta_info(&child, source);
                (name, value)
            })
            .collect();

        let default = meta_infos.remove("default");
        let enum_values = if type_ == ValueType::Enum {
            meta_infos
                .remove("enum_values")
                .map(|value| value.as_array_of_enum())
        } else {
            None
        };

        Self {
            name,
            type_,
            default,
            enum_values,
        }
    }

    pub const fn name(&self) -> &str {
        self.name.as_str()
    }
}

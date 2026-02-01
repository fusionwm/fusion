#![allow(dead_code)]
#![allow(unused)]

mod section;
mod section_option;
mod value;
mod value_type;

use std::str::FromStr;

pub use section::*;
pub use section_option::*;
pub use value::*;
pub use value_type::*;

use tree_sitter::{Node, Parser};

use crate::config::section::Section;

#[must_use]
pub fn read_meta_name<'a>(node: &'a Node, source: &'a str) -> &'a str {
    node.child_by_field_name("name")
        .unwrap()
        .utf8_text(source.as_bytes())
        .unwrap()
}

#[must_use]
pub fn read_meta_info(node: &Node, source: &str) -> Value {
    //field("type", choice($.enum_with_block, $.values)),
    Value::from_str(
        node.child_by_field_name("type")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
    )
    .unwrap()
}

unsafe extern "C" {
    fn tree_sitter_config() -> tree_sitter::Language;
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    sections: Vec<Section>,
}

impl Config {
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let language = unsafe { tree_sitter_config() };
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("Error loading grammar");
        let tree = parser.parse(source, None).expect("Parse failed");

        let mut sections = vec![];

        let root = tree.root_node();
        for child in root.children(&mut root.walk()) {
            if child.kind() == "section" {
                sections.push(Section::from_node(&child, source));
            } else {
                panic!("Unexpected node type: {}", child.kind());
            }
        }

        Config { sections }
    }

    #[must_use]
    pub fn get_value(&self, path: &str) -> Option<Value> {
        Some(Value::String(Str(path.to_string())))
    }
}

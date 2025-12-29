mod section;
mod section_option;
mod value;
mod value_type;

pub use section::*;
pub use section_option::*;
pub use value::*;
pub use value_type::*;

use tree_sitter::{Node, Parser};

use crate::module::config::section::Section;

pub fn read_meta_name<'a>(node: &'a Node, source: &'a str) -> &'a str {
    node.child_by_field_name("name")
        .unwrap()
        .utf8_text(source.as_bytes())
        .unwrap()
}

pub fn read_meta_info(node: &Node, source: &str) -> Value {
    //field("type", choice($.enum_with_block, $.values)),
    Value::from_str(
        node.child_by_field_name("type")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
    )
}

unsafe extern "C" {
    fn tree_sitter_config() -> tree_sitter::Language;
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    sections: Vec<Section>,
}

impl Config {
    #[allow(unused)]
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

    pub fn get_value(&self, path: &str) -> Option<Value> {
        Some(Value::from_str("993"))
    }
}

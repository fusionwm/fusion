use tree_sitter::Node;

use crate::config::section_option::SectionOption;

#[derive(Debug, Clone)]
pub struct Section {
    name: String,
    sections: Vec<Section>,
    options: Vec<SectionOption>,
}

impl Section {
    pub fn from_node(node: &Node, source: &str) -> Self {
        let name = node
            .child_by_field_name("name")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap()
            .to_string();

        let mut sections = vec![];
        let mut options = vec![];

        for child in node.children(&mut node.walk()).skip(1) {
            if child.kind() == "section" {
                sections.push(Self::from_node(&child, source));
            } else if child.kind() == "option" {
                options.push(SectionOption::from_node(&child, source));
            }
        }

        Self {
            name,
            sections,
            options,
        }
    }
}

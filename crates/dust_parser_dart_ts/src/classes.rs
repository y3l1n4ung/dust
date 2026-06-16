mod class_decl;
mod constructors;
pub(crate) mod fields;
pub(crate) mod methods;
mod parse_text;

use dust_parser_dart::ParsedClassSurface;
use dust_text::SourceText;
use tree_sitter::Node;

pub(crate) fn extract_classes(root: Node<'_>, source: &SourceText) -> Vec<ParsedClassSurface> {
    let mut classes = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        if child.kind() == "class_declaration" {
            classes.push(class_decl::extract_class(child, source));
        }
    }

    classes
}

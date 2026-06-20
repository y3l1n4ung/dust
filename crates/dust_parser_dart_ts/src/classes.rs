/// Class declaration extraction from tree-sitter nodes.
mod class_decl;
/// Constructor extraction from class members.
mod constructors;
/// Field extraction shared by classes and mixins.
pub(crate) mod fields;
/// Method and parameter extraction shared by classes and top-level functions.
pub(crate) mod methods;
/// Source-text helpers for parameter parsing.
mod parse_text;

use dust_parser_dart::ParsedClassSurface;
use dust_text::SourceText;
use tree_sitter::Node;

/// Extracts all class declarations from a parsed Dart compilation unit.
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

use dust_parser_dart::ParsedAnnotation;
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{node_text, text_range};

pub(crate) fn extract_annotation(node: Node<'_>, source: &SourceText) -> ParsedAnnotation {
    let mut name = String::new();
    let mut arguments_source = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        match child.kind() {
            "identifier" if name.is_empty() => name = node_text(child, source),
            "annotation_arguments" => arguments_source = Some(node_text(child, source)),
            _ => {}
        }
    }

    ParsedAnnotation {
        name,
        arguments_source,
        span: text_range(node),
    }
}

pub(crate) fn extract_member_annotations(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedAnnotation> {
    let mut annotations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }
    annotations
}

use dust_parser_dart::{ParsedEnumSurface, ParsedEnumVariantSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::extract_annotation,
    syntax::{node_text, text_range},
};

pub(crate) fn extract_enums(root: Node<'_>, source: &SourceText) -> Vec<ParsedEnumSurface> {
    let mut enums = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        if child.kind() == "enum_declaration" {
            enums.push(extract_enum(child, source));
        }
    }
    enums
}

fn extract_enum(node: Node<'_>, source: &SourceText) -> ParsedEnumSurface {
    let name = node
        .child_by_field_name("name")
        .map(|name| node_text(name, source))
        .unwrap_or_default();

    let mut annotations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }

    let mut variants = Vec::new();
    if let Some(body) = node.child_by_field_name("body") {
        let mut body_cursor = body.walk();
        for member in body
            .children(&mut body_cursor)
            .filter(|child| child.is_named())
        {
            if member.kind() == "enum_constant" {
                variants.push(extract_enum_variant(member, source));
            }
        }
    }

    ParsedEnumSurface {
        name,
        annotations,
        variants,
        span: text_range(node),
    }
}

fn extract_enum_variant(node: Node<'_>, source: &SourceText) -> ParsedEnumVariantSurface {
    let name = node
        .child_by_field_name("name")
        .map(|name| node_text(name, source))
        .unwrap_or_default();
    let mut annotations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }
    ParsedEnumVariantSurface {
        name,
        annotations,
        span: text_range(node),
    }
}

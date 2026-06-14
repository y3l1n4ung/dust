use dust_parser_dart::ParameterKind;
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{direct_named_child, node_text};

pub(super) fn determine_parameter_kind(node: Node<'_>, source: &SourceText) -> ParameterKind {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "optional_formal_parameters" {
            let text = node_text(parent, source);
            return if text.trim_start().starts_with('{') {
                ParameterKind::Named
            } else {
                ParameterKind::Positional
            };
        }
        current = parent.parent();
    }

    ParameterKind::Positional
}

pub(super) fn parameter_name_node<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    if let Some(name) = node.child_by_field_name("name") {
        return Some(name);
    }

    if matches!(node.kind(), "default_formal_parameter") {
        return direct_named_child(node, "formal_parameter").and_then(parameter_name_node);
    }

    for wrapper in [
        "formal_parameter",
        "constructor_param",
        "super_formal_parameter",
    ] {
        if let Some(child) = direct_named_child(node, wrapper) {
            if let Some(name) = parameter_name_node(child) {
                return Some(name);
            }
        }
    }

    direct_named_child(node, "identifier")
}

pub(super) fn default_value_source(node: Node<'_>, source: &SourceText) -> Option<String> {
    let parent = node.parent()?;

    let mut found_parameter = false;
    let mut found_separator = false;
    let mut cursor = parent.walk();
    for child in parent.children(&mut cursor) {
        if same_node(child, node) {
            found_parameter = true;
            continue;
        }
        if !found_parameter {
            continue;
        }

        match child.kind() {
            "=" | ":" => found_separator = true,
            "," | "}" | "]" if !found_separator => return None,
            "," | "}" | "]" => return None,
            _ if found_separator && child.is_named() => {
                let value = node_text(child, source);
                return (!value.trim().is_empty()).then_some(value);
            }
            _ => {}
        }
    }

    None
}

fn same_node(left: Node<'_>, right: Node<'_>) -> bool {
    left.kind() == right.kind()
        && left.start_byte() == right.start_byte()
        && left.end_byte() == right.end_byte()
}

use dust_parser_dart::ParameterKind;
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{direct_named_child, node_text};

/// Determines whether an optional parameter group is named or positional.
pub(super) fn optional_parameter_kind(node: Node<'_>, source: &SourceText) -> ParameterKind {
    let bytes = source.as_str().as_bytes();
    let mut index = node.start_byte();
    let end = node.end_byte();
    while index < end && bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }

    if bytes.get(index) == Some(&b'{') {
        ParameterKind::Named
    } else {
        ParameterKind::Positional
    }
}

/// Finds the identifier node that names a formal parameter.
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

/// Returns whether a parameter is a Dart field-formal or super-formal parameter.
pub(super) fn is_field_formal_parameter(node: Node<'_>) -> bool {
    direct_named_child(node, "constructor_param").is_some()
        || direct_named_child(node, "super_formal_parameter").is_some()
        || direct_named_child(node, "formal_parameter").is_some_and(is_field_formal_parameter)
}

/// Returns whether a parameter has the explicit Dart `required` modifier.
pub(super) fn is_required_parameter(node: Node<'_>, source: &SourceText) -> bool {
    let Some(name) = parameter_name_node(node) else {
        return false;
    };
    let before_name = &source.as_str()[node.start_byte()..name.start_byte()];
    if before_name
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|word| word == "required")
    {
        return true;
    }

    let Some(parent) = node.parent() else {
        return false;
    };
    let mut cursor = parent.walk();
    let mut required_since_separator = false;
    for child in parent.children(&mut cursor) {
        if same_node(child, node) {
            return required_since_separator;
        }
        if matches!(child.kind(), "," | "{" | "[") {
            required_since_separator = false;
            continue;
        }
        if child.kind() == "required" || node_text(child, source).trim() == "required" {
            required_since_separator = true;
        }
    }
    false
}

/// Extracts a parameter default-value expression from the surrounding syntax.
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

/// Compares tree-sitter nodes by stable kind and byte range.
fn same_node(left: Node<'_>, right: Node<'_>) -> bool {
    left.kind() == right.kind()
        && left.start_byte() == right.start_byte()
        && left.end_byte() == right.end_byte()
}

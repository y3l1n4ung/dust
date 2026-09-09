//! Walking a declaration's tree-sitter node for annotations, names and types.

use super::*;

/// Extracts annotation children directly attached to a declaration.
pub(super) fn direct_annotations(node: Node<'_>, source: &SourceText) -> Vec<ParsedAnnotation> {
    let mut annotations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }
    annotations
}

/// Finds a direct child with the requested tree-sitter kind.
pub(super) fn direct_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == kind)
}

/// Finds a direct named child with the requested kind before deeper traversal.
pub(super) fn direct_named_child_before<'tree>(
    node: Node<'tree>,
    kind: &str,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.is_named() && child.kind() == kind)
}

/// Finds the last type identifier before a byte boundary.
pub(super) fn first_type_identifier_before<'tree>(
    node: Node<'tree>,
    boundary_byte: usize,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| {
            child.is_named()
                && child.kind() == "type_identifier"
                && child.start_byte() < boundary_byte
        })
        .last()
}

/// Alias for finding a previous type identifier before a byte boundary.
pub(super) fn previous_type_identifier<'tree>(
    node: Node<'tree>,
    boundary_byte: usize,
) -> Option<Node<'tree>> {
    first_type_identifier_before(node, boundary_byte)
}

/// Finds the first type node after a byte boundary.
pub(super) fn first_type_node_after<'tree>(
    node: Node<'tree>,
    boundary_byte: usize,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|child| {
        child.is_named() && is_type_node_kind(child.kind()) && child.start_byte() >= boundary_byte
    })
}

/// Returns whether a tree-sitter kind can begin a Dart type source.
pub(super) fn is_type_node_kind(kind: &str) -> bool {
    matches!(
        kind,
        "type_identifier" | "void_type" | "function_type" | "record_type"
    )
}

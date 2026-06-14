use dust_text::{SourceText, TextRange, TextSize};
use tree_sitter::Node;

pub(crate) fn text_range(node: Node<'_>) -> TextRange {
    TextRange::new(
        TextSize::new(node.start_byte() as u32),
        TextSize::new(node.end_byte() as u32),
    )
}

pub(crate) fn node_text(node: Node<'_>, source: &SourceText) -> String {
    source
        .slice(text_range(node))
        .unwrap_or_default()
        .to_owned()
}

pub(crate) fn first_non_annotation_named_child<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.is_named() && child.kind() != "annotation")
}

pub(crate) fn direct_named_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.is_named() && child.kind() == kind)
}

pub(crate) fn has_direct_child_kind(node: Node<'_>, kind: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| child.kind() == kind)
}

pub(crate) fn find_first_descendant<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    find_first_descendant_by(node, |candidate| candidate.kind() == kind)
}

pub(crate) fn find_first_descendant_by<'tree>(
    node: Node<'tree>,
    predicate: impl Copy + Fn(Node<'tree>) -> bool,
) -> Option<Node<'tree>> {
    if predicate(node) {
        return Some(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_first_descendant_by(child, predicate) {
            return Some(found);
        }
    }

    None
}

pub(crate) fn has_descendant_kind(node: Node<'_>, kind: &str) -> bool {
    find_first_descendant(node, kind).is_some()
}

pub(crate) fn find_first_descendant_text(
    node: Node<'_>,
    source: &SourceText,
    kinds: &[&str],
) -> Option<String> {
    if node.is_named() && kinds.iter().any(|kind| node.kind() == *kind) {
        return Some(node_text(node, source));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_first_descendant_text(child, source, kinds) {
            return Some(found);
        }
    }

    None
}

pub(crate) fn unquote(text: String) -> String {
    text.trim().trim_matches('\'').trim_matches('"').to_owned()
}

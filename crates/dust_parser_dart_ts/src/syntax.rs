use dust_text::{SourceText, TextRange, TextSize};
use tree_sitter::Node;

pub(crate) fn class_header_text(node: Node<'_>, source: &SourceText) -> String {
    let body_start = node
        .child_by_field_name("body")
        .map(|body| body.start_byte())
        .unwrap_or_else(|| node.end_byte());
    let range = TextRange::new(
        TextSize::new(node.start_byte() as u32),
        TextSize::new(body_start as u32),
    );
    source.slice(range).unwrap_or_default().to_owned()
}

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

pub(crate) fn find_last_descendant_text(
    node: Node<'_>,
    source: &SourceText,
    kinds: &[&str],
) -> Option<String> {
    let mut values = Vec::new();
    collect_descendant_texts(node, source, kinds, &mut values);
    values.pop()
}

pub(crate) fn unquote(text: String) -> String {
    text.trim().trim_matches('\'').trim_matches('"').to_owned()
}

fn collect_descendant_texts(
    node: Node<'_>,
    source: &SourceText,
    kinds: &[&str],
    out: &mut Vec<String>,
) {
    if node.is_named() && kinds.iter().any(|kind| node.kind() == *kind) {
        out.push(node_text(node, source));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_descendant_texts(child, source, kinds, out);
    }
}

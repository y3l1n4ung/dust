use dust_parser_dart::{ParsedTypeKind, ParsedTypeSurface};
use dust_text::{SourceText, TextRange, TextSize};
use tree_sitter::Node;

/// Extracts one type surface before the named declaration item at `boundary_byte`.
pub(crate) fn extract_type_before(
    node: Node<'_>,
    boundary_byte: usize,
    source: &SourceText,
) -> Option<ParsedTypeSurface> {
    let type_node = first_type_node_before(node, boundary_byte)?;
    type_surface_from_node(type_node, boundary_byte, source)
}

/// Extracts one type surface from a tree-sitter type node.
pub(crate) fn extract_type_node(
    type_node: Node<'_>,
    boundary_byte: usize,
    source: &SourceText,
) -> Option<ParsedTypeSurface> {
    type_surface_from_node(type_node, boundary_byte, source)
}

fn type_surface_from_node(
    type_node: Node<'_>,
    boundary_byte: usize,
    source: &SourceText,
) -> Option<ParsedTypeSurface> {
    let type_start = type_node.start_byte();
    let raw = source.as_str().get(type_start..boundary_byte)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains("this.") || trimmed.contains("super.") {
        return None;
    }

    let leading = raw.len().saturating_sub(raw.trim_start().len());
    let trailing = raw.trim_end().len();
    let start = type_start + leading;
    let end = type_start + trailing;
    let span = TextRange::new(TextSize::new(start as u32), TextSize::new(end as u32));
    let nullable = trimmed.ends_with('?');
    let kind = type_kind_from_node(type_node, end, source)
        .or_else(|| ParsedTypeSurface::parse(trimmed, span).map(|parsed| parsed.kind))?;
    Some(ParsedTypeSurface {
        source: trimmed.to_owned(),
        span,
        kind,
        nullable,
    })
}

fn type_kind_from_node(
    type_node: Node<'_>,
    type_end_byte: usize,
    source: &SourceText,
) -> Option<ParsedTypeKind> {
    match type_node.kind() {
        "function_type" => Some(ParsedTypeKind::Function),
        "record_type" => Some(ParsedTypeKind::Record),
        "void_type" => Some(ParsedTypeKind::Named {
            name: "void".to_owned(),
            args: Vec::new(),
        }),
        "type_identifier" => {
            let args_node = find_type_arguments_after(type_node, type_end_byte);
            let name = named_type_source(type_node, args_node, type_end_byte, source)?;
            if name == "dynamic" {
                return Some(ParsedTypeKind::Dynamic);
            }
            if is_builtin(&name) {
                return Some(ParsedTypeKind::Builtin(name));
            }
            Some(ParsedTypeKind::Named {
                name,
                args: args_node
                    .map(|args| extract_type_arguments(args, source))
                    .unwrap_or_default(),
            })
        }
        _ => None,
    }
}

fn named_type_source(
    type_node: Node<'_>,
    args_node: Option<Node<'_>>,
    type_end_byte: usize,
    source: &SourceText,
) -> Option<String> {
    let end = args_node
        .map(|node| node.start_byte())
        .unwrap_or(type_end_byte);
    let source = source.as_str().get(type_node.start_byte()..end)?;
    let source = source.trim().trim_end_matches('?').trim();
    (!source.is_empty()).then(|| source.to_owned())
}

fn find_type_arguments_after<'tree>(
    type_node: Node<'tree>,
    type_end_byte: usize,
) -> Option<Node<'tree>> {
    let mut container = type_node.parent();
    while let Some(node) = container {
        if node.start_byte() > type_node.start_byte() || node.end_byte() < type_end_byte {
            container = node.parent();
            continue;
        }

        let mut cursor = node.walk();
        if let Some(args) = node.children(&mut cursor).find(|child| {
            child.kind() == "type_arguments"
                && child.start_byte() >= type_node.end_byte()
                && child.end_byte() <= type_end_byte
        }) {
            return Some(args);
        }
        container = node.parent();
    }
    None
}

fn extract_type_arguments(args_node: Node<'_>, source: &SourceText) -> Vec<ParsedTypeSurface> {
    let mut args = Vec::new();
    let mut current_start = None;
    let mut current_end = None;
    let mut cursor = args_node.walk();

    for child in args_node.children(&mut cursor) {
        match child.kind() {
            "<" => {
                current_start = None;
                current_end = None;
            }
            "," | ">" => {
                if let Some(arg) =
                    type_argument_from_range(args_node, current_start, current_end, source)
                {
                    args.push(arg);
                }
                current_start = None;
                current_end = None;
            }
            _ => {
                if child.is_named() && current_start.is_none() {
                    current_start = Some(child.start_byte());
                }
                if current_start.is_some() {
                    current_end = Some(child.end_byte());
                }
            }
        }
    }

    args
}

fn type_argument_from_range(
    args_node: Node<'_>,
    start: Option<usize>,
    end: Option<usize>,
    source: &SourceText,
) -> Option<ParsedTypeSurface> {
    let start = start?;
    let end = end?;
    let type_node = first_type_node_in_range(args_node, start, end)?;
    type_surface_from_node(type_node, end, source)
}

fn first_type_node_in_range<'tree>(
    node: Node<'tree>,
    start_byte: usize,
    end_byte: usize,
) -> Option<Node<'tree>> {
    if node.end_byte() <= start_byte || node.start_byte() >= end_byte {
        return None;
    }
    if is_type_root(node.kind()) && node.start_byte() >= start_byte && node.start_byte() < end_byte
    {
        return Some(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if let Some(found) = first_type_node_in_range(child, start_byte, end_byte) {
            return Some(found);
        }
    }

    None
}

fn first_type_node_before<'tree>(node: Node<'tree>, boundary_byte: usize) -> Option<Node<'tree>> {
    if node.start_byte() >= boundary_byte {
        return None;
    }
    if node.kind() == "annotation" {
        return None;
    }
    if is_type_root(node.kind()) {
        return Some(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if let Some(found) = first_type_node_before(child, boundary_byte) {
            return Some(found);
        }
    }

    None
}

fn is_type_root(kind: &str) -> bool {
    matches!(
        kind,
        "type_identifier" | "void_type" | "function_type" | "record_type"
    )
}

fn is_builtin(source: &str) -> bool {
    matches!(
        source,
        "String" | "int" | "bool" | "double" | "num" | "Object"
    )
}

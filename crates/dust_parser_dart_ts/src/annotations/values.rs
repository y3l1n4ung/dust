use dust_dart_syntax::parse_string_literal;
use dust_parser_dart::{
    ParsedAnnotationNumberKind, ParsedAnnotationValue, ParsedAnnotationValueRootKind,
};
use dust_text::{SourceText, TextRange};
use tree_sitter::Node;

use crate::syntax::{direct_named_child, has_descendant_kind, node_text, text_range};

/// Converts an annotation argument container into a parser-owned value.
pub(super) fn annotation_value_from_container(
    container: Node<'_>,
    value_source: String,
    value_span: TextRange,
    source: &SourceText,
) -> Option<ParsedAnnotationValue> {
    let value_node = annotation_argument_value_node(container, value_span)?;
    Some(annotation_value(
        value_node,
        source,
        value_source,
        value_span,
        is_member_selector_chain(container),
    ))
}

/// Returns the expression node inside one annotation argument container.
fn annotation_argument_value_node(node: Node<'_>, value_span: TextRange) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() != "label")
        .find(|child| {
            let child_range = text_range(*child);
            child_range.start() >= value_span.start() && child_range.end() <= value_span.end()
        })
}

/// Converts a tree-sitter value node into a parser-owned annotation value.
fn annotation_value(
    node: Node<'_>,
    source: &SourceText,
    value_source: String,
    value_span: TextRange,
    force_member: bool,
) -> ParsedAnnotationValue {
    let kind = if force_member {
        ParsedAnnotationValueRootKind::Member(value_source.clone())
    } else {
        match node.kind() {
            "null_literal" => ParsedAnnotationValueRootKind::Null,
            "true" => ParsedAnnotationValueRootKind::Bool(true),
            "false" => ParsedAnnotationValueRootKind::Bool(false),
            "decimal_integer_literal" | "hex_integer_literal" => {
                ParsedAnnotationValueRootKind::Number(ParsedAnnotationNumberKind::Int)
            }
            "decimal_floating_point_literal" => {
                ParsedAnnotationValueRootKind::Number(ParsedAnnotationNumberKind::Double)
            }
            "string_literal" => ParsedAnnotationValueRootKind::String(
                parse_string_literal(&value_source).unwrap_or_else(|| value_source.clone()),
            ),
            "list_literal" => ParsedAnnotationValueRootKind::List,
            "set_or_map_literal" => set_or_map_kind(node, source),
            "record_literal" => ParsedAnnotationValueRootKind::Record,
            "const_object_expression" | "constructor_invocation" => {
                ParsedAnnotationValueRootKind::Constructor {
                    name: constructor_name(node, source).unwrap_or_else(|| value_source.clone()),
                }
            }
            "identifier" | "qualified" | "selector" => {
                ParsedAnnotationValueRootKind::Member(value_source.clone())
            }
            _ => ParsedAnnotationValueRootKind::Expression,
        }
    };

    ParsedAnnotationValue {
        source: value_source,
        span: value_span,
        kind,
    }
}

/// Returns whether the value is an identifier followed only by selectors.
fn is_member_selector_chain(node: Node<'_>) -> bool {
    let mut cursor = node.walk();
    let mut saw_base = false;
    let mut saw_selector = false;

    for child in node
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() != "label")
    {
        if !saw_base {
            if child.kind() != "identifier" && child.kind() != "qualified" {
                return false;
            }
            saw_base = true;
            continue;
        }

        if child.kind() != "selector" {
            return false;
        }
        saw_selector = true;
    }

    saw_base && saw_selector
}

/// Classifies Dart's shared set/map literal node.
fn set_or_map_kind(node: Node<'_>, source: &SourceText) -> ParsedAnnotationValueRootKind {
    if has_descendant_kind(node, ":") {
        return ParsedAnnotationValueRootKind::Map;
    }

    let value_source = node_text(node, source);
    if type_argument_source(node, source).is_some_and(|args| args.contains(',')) {
        return ParsedAnnotationValueRootKind::Map;
    }
    if literal_body(&value_source).is_empty() && type_argument_source(node, source).is_none() {
        return ParsedAnnotationValueRootKind::Map;
    }

    ParsedAnnotationValueRootKind::Set
}

/// Returns the source inside a collection literal's braces.
fn literal_body(source: &str) -> &str {
    let Some((_, rest)) = source.split_once('{') else {
        return "";
    };
    rest.rsplit_once('}').map_or("", |(body, _)| body).trim()
}

/// Returns direct type argument source for a value node.
fn type_argument_source<'a>(node: Node<'_>, source: &'a SourceText) -> Option<&'a str> {
    let args = direct_named_child(node, "type_arguments")?;
    source.slice(text_range(args))
}

/// Returns constructor/type source for a constructor expression.
fn constructor_name(node: Node<'_>, source: &SourceText) -> Option<String> {
    let expression_name = node_text(node, source)
        .split_once('(')
        .map(|(name, _)| strip_const_keyword(name).trim().to_owned())
        .filter(|name| !name.is_empty());
    if expression_name.is_some() {
        return expression_name;
    }

    if let Some(type_node) = node.child_by_field_name("type") {
        return Some(node_text(type_node, source));
    }
    if let Some(constructor_node) = node.child_by_field_name("constructor") {
        return Some(node_text(constructor_node, source));
    }
    None
}

/// Removes a leading `const` keyword without touching identifiers.
fn strip_const_keyword(source: &str) -> &str {
    let trimmed = source.trim_start();
    let Some(rest) = trimmed.strip_prefix("const") else {
        return trimmed;
    };
    if rest.as_bytes().first().is_some_and(u8::is_ascii_whitespace) {
        return rest.trim_start();
    }
    trimmed
}

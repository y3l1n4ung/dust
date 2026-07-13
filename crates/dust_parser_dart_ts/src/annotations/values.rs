use dust_dart_syntax::parse_string_literal;
use dust_parser_dart::{
    ParsedAnnotationNumberKind, ParsedAnnotationValue, ParsedAnnotationValueRootKind,
};
use dust_text::{SourceText, TextRange};
use tree_sitter::Node;

use super::extract_annotation_arguments;
use crate::syntax::{direct_named_child, has_descendant_kind, node_text, text_range};

/// Converts an annotation argument container into a parser-owned value.
pub(super) fn annotation_value_from_container(
    container: Node<'_>,
    value_source: String,
    value_span: TextRange,
    source: &SourceText,
) -> Option<ParsedAnnotationValue> {
    if is_member_selector_chain(container) && has_descendant_kind(container, "arguments") {
        let name = value_source
            .split_once('(')
            .map_or(value_source.as_str(), |(name, _)| name)
            .trim()
            .to_owned();
        return Some(ParsedAnnotationValue {
            source: value_source,
            span: value_span,
            kind: ParsedAnnotationValueRootKind::Constructor {
                name,
                arguments: constructor_arguments(container, source),
            },
        });
    }

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
            "unary_expression" => signed_number_kind(node)
                .map(ParsedAnnotationValueRootKind::Number)
                .unwrap_or(ParsedAnnotationValueRootKind::Expression),
            "string_literal" => ParsedAnnotationValueRootKind::String(
                parse_string_literal(&value_source).unwrap_or_else(|| value_source.clone()),
            ),
            "list_literal" => ParsedAnnotationValueRootKind::List(collection_values(node, source)),
            "set_or_map_literal" => set_or_map_kind(node, source),
            "record_literal" => ParsedAnnotationValueRootKind::Record,
            "const_object_expression" | "constructor_invocation" => {
                ParsedAnnotationValueRootKind::Constructor {
                    name: constructor_name(node, source).unwrap_or_else(|| value_source.clone()),
                    arguments: constructor_arguments(node, source),
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

/// Returns the numeric kind for a directly signed numeric literal.
fn signed_number_kind(node: Node<'_>) -> Option<ParsedAnnotationNumberKind> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.is_named())
        .find_map(|child| match child.kind() {
            "decimal_integer_literal" | "hex_integer_literal" => {
                Some(ParsedAnnotationNumberKind::Int)
            }
            "decimal_floating_point_literal" => Some(ParsedAnnotationNumberKind::Double),
            _ => None,
        })
}

/// Parses direct collection elements without reparsing their source text.
fn collection_values(node: Node<'_>, source: &SourceText) -> Vec<ParsedAnnotationValue> {
    let mut cursor = node.walk();
    let children = node
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() != "type_arguments")
        .collect::<Vec<_>>();
    let mut values = Vec::new();
    let mut index = 0;

    while let Some(child) = children.get(index).copied() {
        if matches!(child.kind(), "identifier" | "qualified") {
            let mut end = index + 1;
            while children
                .get(end)
                .is_some_and(|selector| selector.kind() == "selector")
            {
                end += 1;
            }
            if end > index + 1 {
                let last = children[end - 1];
                let span = TextRange::new(text_range(child).start(), text_range(last).end());
                let value_source = source.slice(span).unwrap_or_default().to_owned();
                let kind = if value_source.contains('(') {
                    ParsedAnnotationValueRootKind::Constructor {
                        name: value_source
                            .split_once('(')
                            .map_or(value_source.as_str(), |(name, _)| name)
                            .trim()
                            .to_owned(),
                        arguments: constructor_arguments(last, source),
                    }
                } else {
                    ParsedAnnotationValueRootKind::Member(value_source.clone())
                };
                values.push(ParsedAnnotationValue {
                    source: value_source,
                    span,
                    kind,
                });
                index = end;
                continue;
            }
        }

        let span = text_range(child);
        values.push(annotation_value(
            child,
            source,
            node_text(child, source),
            span,
            false,
        ));
        index += 1;
    }

    values
}

/// Extracts structured arguments from a constructor node or selector.
fn constructor_arguments(
    node: Node<'_>,
    source: &SourceText,
) -> Box<dust_parser_dart::ParsedAnnotationArguments> {
    let arguments = node
        .child_by_field_name("arguments")
        .or_else(|| crate::syntax::find_first_descendant(node, "arguments"));
    Box::new(arguments.map_or_else(Default::default, |arguments| {
        extract_annotation_arguments(arguments, source)
    }))
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

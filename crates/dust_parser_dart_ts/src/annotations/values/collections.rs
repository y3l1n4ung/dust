//! Reading list, set, map and constructor argument values out of an annotation.

use super::*;

/// Parses direct key/value pairs from a map literal.
pub(super) fn map_values(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<(ParsedAnnotationValue, ParsedAnnotationValue)> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "pair")
        .filter_map(|pair| {
            let mut pair_cursor = pair.walk();
            let values = pair
                .children(&mut pair_cursor)
                .filter(|child| child.is_named())
                .collect::<Vec<_>>();
            let [key, value, ..] = values.as_slice() else {
                return None;
            };
            Some((
                annotation_value(
                    *key,
                    source,
                    node_text(*key, source),
                    text_range(*key),
                    false,
                ),
                annotation_value(
                    *value,
                    source,
                    node_text(*value, source),
                    text_range(*value),
                    false,
                ),
            ))
        })
        .collect()
}

/// Parses named fields from a record literal.
pub(super) fn record_values(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<(String, ParsedAnnotationValue)> {
    let mut cursor = node.walk();
    let children = node
        .children(&mut cursor)
        .filter(|child| child.is_named())
        .collect::<Vec<_>>();
    let mut fields = Vec::new();
    let mut index = 0;

    while let Some(label) = children.get(index).copied() {
        if label.kind() != "label" {
            index += 1;
            continue;
        }
        let name = node_text(label, source)
            .trim_end_matches(':')
            .trim()
            .to_owned();
        let Some(value) = children.get(index + 1).copied() else {
            break;
        };
        fields.push((
            name,
            annotation_value(
                value,
                source,
                node_text(value, source),
                text_range(value),
                false,
            ),
        ));
        index += 2;
    }

    fields
}

/// Extracts structured arguments from a constructor node or selector.
pub(super) fn constructor_arguments(
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
pub(super) fn is_member_selector_chain(node: Node<'_>) -> bool {
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
pub(super) fn set_or_map_kind(
    node: Node<'_>,
    source: &SourceText,
) -> ParsedAnnotationValueRootKind {
    let has_pairs = {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .any(|child| child.is_named() && child.kind() == "pair")
    };
    if has_pairs {
        return ParsedAnnotationValueRootKind::Map(map_values(node, source));
    }

    let value_source = node_text(node, source);
    if type_argument_source(node, source).is_some_and(|args| args.contains(',')) {
        return ParsedAnnotationValueRootKind::Map(map_values(node, source));
    }
    if literal_body(&value_source).is_empty() && type_argument_source(node, source).is_none() {
        return ParsedAnnotationValueRootKind::Map(map_values(node, source));
    }

    ParsedAnnotationValueRootKind::Set(set_values(node, source))
}

/// Returns the source inside a collection literal's braces.
pub(super) fn literal_body(source: &str) -> &str {
    let Some((_, rest)) = source.split_once('{') else {
        return "";
    };
    rest.rsplit_once('}').map_or("", |(body, _)| body).trim()
}

/// Returns direct type argument source for a value node.
pub(super) fn type_argument_source<'a>(node: Node<'_>, source: &'a SourceText) -> Option<&'a str> {
    let args = direct_named_child(node, "type_arguments")?;
    source.slice(text_range(args))
}

/// Returns constructor/type source for a constructor expression.
pub(super) fn constructor_name(node: Node<'_>, source: &SourceText) -> Option<String> {
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
pub(super) fn strip_const_keyword(source: &str) -> &str {
    let trimmed = source.trim_start();
    let Some(rest) = trimmed.strip_prefix("const") else {
        return trimmed;
    };
    if rest.as_bytes().first().is_some_and(u8::is_ascii_whitespace) {
        return rest.trim_start();
    }
    trimmed
}

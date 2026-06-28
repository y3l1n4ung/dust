use dust_dart_syntax::parse_static_dart_string_literal;
use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_text::{SourceText, TextRange, TextSize};
use tree_sitter::{Node, Query, QueryMatch};

use crate::syntax::node_text;

use super::{I18nTranslationKind, I18nTranslationUse};

/// Lowers one query match into an i18n entry.
pub(super) fn lower_match(
    source: &SourceText,
    query: &Query,
    query_match: &QueryMatch<'_, '_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<I18nTranslationUse> {
    let call = capture_node(query, query_match, "call")?;
    let arguments = capture_node(query, query_match, "arguments")?;
    let call_shape = call_shape(call, source)?;
    lower_call(source, arguments, call_shape, diagnostics)
}

/// Lowers one matched call shape.
fn lower_call(
    source: &SourceText,
    arguments: Node<'_>,
    call_shape: I18nCallShape,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<I18nTranslationUse> {
    let first_argument = first_positional_argument(arguments)?;
    let Some(key) = static_string(first_argument, source) else {
        diagnostics.push(non_literal_key_diagnostic(
            source,
            call_shape.kind,
            call_shape.span,
        ));
        return None;
    };

    let default_text = named_argument_value(arguments, source, "defaultText")
        .and_then(|value| static_string(value, source));
    let args = named_argument_value(arguments, source, "args")
        .map_or_else(Vec::new, |value| placeholder_keys(value, source));

    Some(I18nTranslationUse {
        namespace: namespace_for(&key),
        key,
        default_text,
        args,
        kind: call_shape.kind,
        span: call_shape.span,
    })
}

/// Resolves the public i18n API represented by one matched call node.
fn call_shape(call: Node<'_>, source: &SourceText) -> Option<I18nCallShape> {
    if call.kind() == "const_object_expression" {
        return const_object_call_shape(call, source);
    }
    selector_call_shape(call, source)
}

/// Resolves `const TranslatedText(...)`.
fn const_object_call_shape(call: Node<'_>, source: &SourceText) -> Option<I18nCallShape> {
    let type_node = call.child_by_field_name("type")?;
    if node_text(type_node, source) != "TranslatedText" {
        return None;
    }
    if call
        .child_by_field_name("constructor")
        .is_some_and(|constructor| node_text(constructor, source) == "dynamic")
    {
        return None;
    }
    Some(I18nCallShape {
        kind: I18nTranslationKind::TranslatedText,
        span: text_range(call.start_byte(), call.end_byte()),
    })
}

/// Resolves selector calls such as `TranslatedText(...)` and `context.tr(...)`.
fn selector_call_shape(selector: Node<'_>, source: &SourceText) -> Option<I18nCallShape> {
    let previous = selector.prev_named_sibling()?;
    if previous.kind() == "identifier" && node_text(previous, source) == "TranslatedText" {
        return Some(I18nCallShape {
            kind: I18nTranslationKind::TranslatedText,
            span: text_range(previous.start_byte(), selector.end_byte()),
        });
    }

    if previous.kind() != "selector" || selector_property_name(previous, source)? != "tr" {
        return None;
    }
    let receiver = previous.prev_named_sibling()?;
    if receiver.kind() != "identifier" || node_text(receiver, source) != "context" {
        return None;
    }
    Some(I18nCallShape {
        kind: I18nTranslationKind::ContextTr,
        span: text_range(receiver.start_byte(), selector.end_byte()),
    })
}

/// Returns the method/property name carried by a selector.
fn selector_property_name(selector: Node<'_>, source: &SourceText) -> Option<String> {
    let assignable = direct_named_child(selector, "unconditional_assignable_selector")?;
    direct_named_child(assignable, "identifier").map(|identifier| node_text(identifier, source))
}

/// Returns the first positional argument value node.
fn first_positional_argument(arguments: Node<'_>) -> Option<Node<'_>> {
    argument_values(arguments)
        .into_iter()
        .find(|value| value.kind() != "named_argument")
}

/// Returns one named argument value node by label.
fn named_argument_value<'tree>(
    arguments: Node<'tree>,
    source: &SourceText,
    name: &str,
) -> Option<Node<'tree>> {
    argument_values(arguments)
        .into_iter()
        .filter(|value| value.kind() == "named_argument")
        .find_map(|named| {
            (named_argument_label(named, source)? == name)
                .then(|| named_argument_value_node(named))?
        })
}

/// Returns named argument value nodes from an `arguments` node.
fn argument_values(arguments: Node<'_>) -> Vec<Node<'_>> {
    named_children(arguments)
        .into_iter()
        .filter(|child| child.kind() == "argument")
        .filter_map(first_named_child)
        .collect()
}

/// Returns a named argument label.
fn named_argument_label(named: Node<'_>, source: &SourceText) -> Option<String> {
    let label = direct_named_child(named, "label")?;
    direct_named_child(label, "identifier").map(|identifier| node_text(identifier, source))
}

/// Returns a named argument value.
fn named_argument_value_node(named: Node<'_>) -> Option<Node<'_>> {
    named_children(named)
        .into_iter()
        .find(|child| child.kind() != "label")
}

/// Parses a static Dart string literal node.
fn static_string(node: Node<'_>, source: &SourceText) -> Option<String> {
    (node.kind() == "string_literal")
        .then(|| parse_static_dart_string_literal(&node_text(node, source)))?
}

/// Parses placeholder keys from a map literal node.
fn placeholder_keys(node: Node<'_>, source: &SourceText) -> Vec<String> {
    if node.kind() != "set_or_map_literal" {
        return Vec::new();
    }

    let mut keys = Vec::new();
    for pair in named_children(node)
        .into_iter()
        .filter(|child| child.kind() == "pair")
    {
        let Some(key_node) = pair.child_by_field_name("key") else {
            continue;
        };
        let Some(key) = static_string(key_node, source) else {
            continue;
        };
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

/// Builds a warning for a runtime key passed to a static i18n API.
fn non_literal_key_diagnostic(
    source: &SourceText,
    kind: I18nTranslationKind,
    range: TextRange,
) -> Diagnostic {
    let message = match kind {
        I18nTranslationKind::TranslatedText => {
            "TranslatedText requires a string literal key; use TranslatedText.dynamic for runtime keys"
        }
        I18nTranslationKind::ContextTr => "context.tr requires a string literal key",
    };
    Diagnostic::warning(message).with_label(SourceLabel::new(
        source.file_id(),
        range,
        "runtime key used here",
    ))
}

/// Returns the first key segment as the namespace.
fn namespace_for(key: &str) -> String {
    key.split_once('.')
        .or_else(|| key.split_once('_'))
        .map_or_else(String::new, |(namespace, _)| namespace.to_owned())
}

/// Returns one captured query node by name.
fn capture_node<'tree>(
    query: &Query,
    query_match: &QueryMatch<'_, 'tree>,
    name: &str,
) -> Option<Node<'tree>> {
    query_match.captures.iter().find_map(|capture| {
        (query.capture_names().get(capture.index as usize).copied() == Some(name))
            .then_some(capture.node)
    })
}

/// Returns direct named children.
fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.is_named())
        .collect()
}

/// Returns the first named child.
fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|child| child.is_named())
}

/// Returns the first direct named child of one kind.
fn direct_named_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.is_named() && child.kind() == kind)
}

/// Builds a Dust text range from byte offsets.
fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(offset(start), offset(end))
}

/// Converts a byte offset into Dust text size.
fn offset(value: usize) -> TextSize {
    TextSize::new(u32::try_from(value).unwrap_or(u32::MAX))
}

/// Matched i18n call metadata.
struct I18nCallShape {
    /// Recognized public i18n API kind.
    kind: I18nTranslationKind,
    /// Full source span for the call.
    span: TextRange,
}

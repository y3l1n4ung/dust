use dust_parser_dart::{
    ParsedAnnotation, ParsedAnnotationArgument, ParsedAnnotationArguments,
    ParsedAnnotationNamedArgument,
};
use dust_text::{SourceText, TextRange, TextSize};
use tree_sitter::Node;

use crate::syntax::{direct_named_child, find_first_descendant_text, node_text, text_range};

/// Extracts a structured annotation from a tree-sitter annotation node.
pub(crate) fn extract_annotation(node: Node<'_>, source: &SourceText) -> ParsedAnnotation {
    let (name, prefix, qualified_name) = node
        .child_by_field_name("name")
        .map(|name| annotation_name(name, source))
        .unwrap_or_default();
    let mut arguments_source = None;
    let mut parsed_arguments = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation_arguments" {
            arguments_source = Some(node_text(child, source));
            parsed_arguments = Some(extract_annotation_arguments(child, source));
        }
    }

    ParsedAnnotation {
        name,
        prefix,
        qualified_name,
        arguments_source,
        parsed_arguments,
        span: text_range(node),
    }
}

/// Splits an annotation name into short, prefix, and qualified forms.
fn annotation_name(node: Node<'_>, source: &SourceText) -> (String, Option<String>, String) {
    let qualified_name = node_text(node, source);
    if node.kind() != "qualified" {
        return (qualified_name.clone(), None, qualified_name);
    }

    let mut cursor = node.walk();
    let Some(short_node) = node
        .children(&mut cursor)
        .filter(|child| child.is_named())
        .last()
    else {
        return (qualified_name.clone(), None, qualified_name);
    };

    let name = node_text(short_node, source);
    let prefix = source
        .slice(TextRange::new(
            TextSize::new(node.start_byte() as u32),
            TextSize::new(short_node.start_byte() as u32),
        ))
        .map(str::trim)
        .map(|value| value.trim_end_matches('.').trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    (name, prefix, qualified_name)
}

/// Extracts positional and named arguments from an annotation argument list.
fn extract_annotation_arguments(node: Node<'_>, source: &SourceText) -> ParsedAnnotationArguments {
    let mut arguments = ParsedAnnotationArguments::default();
    let mut cursor = node.walk();
    for child in node
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "argument")
    {
        if let Some(named) = direct_named_child(child, "named_argument") {
            if let Some(argument) = extract_named_argument(child, named, source) {
                arguments.named.push(argument);
            }
        } else {
            arguments.positional.push(ParsedAnnotationArgument {
                source: node_text(child, source),
                span: text_range(child),
            });
        }
    }
    arguments
}

/// Extracts one named annotation argument and its value span.
fn extract_named_argument(
    argument_node: Node<'_>,
    named_node: Node<'_>,
    source: &SourceText,
) -> Option<ParsedAnnotationNamedArgument> {
    let label = direct_named_child(named_node, "label")?;
    let name = find_first_descendant_text(label, source, &["identifier"])?;
    let value_span = named_value_span(named_node, label, source);

    Some(ParsedAnnotationNamedArgument {
        name,
        source: node_text(argument_node, source),
        value_source: source.slice(value_span).unwrap_or_default().to_owned(),
        span: text_range(argument_node),
        value_span,
    })
}

/// Returns the source span containing a named argument value without the label.
fn named_value_span(named_node: Node<'_>, label_node: Node<'_>, source: &SourceText) -> TextRange {
    let mut start = label_node.end_byte();
    let mut end = named_node.end_byte();
    let bytes = source.as_str().as_bytes();

    while start < end && bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
        start += 1;
    }
    if bytes.get(start) == Some(&b':') {
        start += 1;
    }
    while start < end && bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
        start += 1;
    }
    while end > start
        && bytes
            .get(end.saturating_sub(1))
            .is_some_and(u8::is_ascii_whitespace)
    {
        end -= 1;
    }

    TextRange::new(TextSize::from(start), TextSize::from(end))
}

/// Extracts annotations attached directly to a class or mixin member node.
pub(crate) fn extract_member_annotations(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedAnnotation> {
    extract_direct_annotations(node, source)
}

/// Extracts annotation children that are direct named children of a node.
pub(crate) fn extract_direct_annotations(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedAnnotation> {
    let mut annotations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }
    annotations
}

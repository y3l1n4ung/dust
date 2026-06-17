use dust_parser_dart::{
    ParsedAnnotation, ParsedAnnotationArgument, ParsedAnnotationArguments,
    ParsedAnnotationNamedArgument,
};
use dust_text::{SourceText, TextRange, TextSize};
use tree_sitter::Node;

use crate::syntax::{find_first_descendant_text, node_text, text_range};

pub(crate) fn extract_annotation(node: Node<'_>, source: &SourceText) -> ParsedAnnotation {
    let mut name = String::new();
    let mut arguments_source = None;
    let mut parsed_arguments = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        match child.kind() {
            "identifier" if name.is_empty() => name = node_text(child, source),
            "annotation_arguments" => {
                arguments_source = Some(node_text(child, source));
                parsed_arguments = Some(extract_annotation_arguments(child, source));
            }
            _ => {}
        }
    }

    ParsedAnnotation {
        name,
        arguments_source,
        parsed_arguments,
        span: text_range(node),
    }
}

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

fn direct_named_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.is_named() && child.kind() == kind)
}

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

pub(crate) fn extract_member_annotations(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedAnnotation> {
    extract_direct_annotations(node, source)
}

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

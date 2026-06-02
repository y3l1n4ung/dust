use dust_parser_dart::{ParsedAnnotation, ParsedFieldSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{find_first_descendant, find_last_descendant_text, node_text, text_range};

use super::parse_text::extract_type_prefix;

pub(super) fn extract_fields(
    node: Node<'_>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Vec<ParsedFieldSurface> {
    let Some(identifier_list) = find_first_descendant(node, "initialized_identifier_list") else {
        return Vec::new();
    };

    let declaration_text = node_text(node, source);
    let relative_type_end = identifier_list
        .start_byte()
        .saturating_sub(node.start_byte());
    let type_source = extract_type_prefix(&declaration_text, relative_type_end);

    let mut fields = Vec::new();
    let mut cursor = identifier_list.walk();
    for initialized in identifier_list
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "initialized_identifier")
    {
        let name =
            find_last_descendant_text(initialized, source, &["identifier"]).unwrap_or_default();
        fields.push(ParsedFieldSurface {
            name,
            annotations: annotations.to_vec(),
            type_source: type_source.clone(),
            has_default: node_text(initialized, source).contains('='),
            span: text_range(initialized),
        });
    }

    fields
}

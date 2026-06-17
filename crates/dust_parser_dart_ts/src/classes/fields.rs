use dust_parser_dart::{ParsedAnnotation, ParsedFieldSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{find_first_descendant, node_text, text_range};
use crate::types::extract_type_before;

pub(crate) fn extract_fields(
    node: Node<'_>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Vec<ParsedFieldSurface> {
    let Some(identifier_list) = find_first_descendant(node, "initialized_identifier_list") else {
        return Vec::new();
    };

    extract_fields_from_identifier_list(node, identifier_list, annotations, source)
}

pub(crate) fn extract_fields_from_identifier_list(
    node: Node<'_>,
    identifier_list: Node<'_>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Vec<ParsedFieldSurface> {
    let parsed_type = extract_type_before(node, identifier_list.start_byte(), source);
    let type_source = parsed_type.as_ref().map(|ty| ty.source.clone());

    let mut fields = Vec::new();
    let mut cursor = identifier_list.walk();
    for initialized in identifier_list
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "initialized_identifier")
    {
        let name = initialized
            .child_by_field_name("name")
            .map(|name| node_text(name, source))
            .unwrap_or_default();
        fields.push(ParsedFieldSurface {
            name,
            annotations: annotations.to_vec(),
            type_source: type_source.clone(),
            parsed_type: parsed_type.clone(),
            has_default: initialized.child_by_field_name("value").is_some(),
            span: text_range(initialized),
        });
    }

    fields
}

//! Extracting extension and extension type declarations.

use super::*;

/// Extracts an extension surface from an extension declaration.
pub(super) fn extract_extension(node: Node<'_>, source: &SourceText) -> ParsedExtensionSurface {
    let name = node
        .child_by_field_name("name")
        .map(|name| node_text(name, source));
    let parsed_on_type = node
        .child_by_field_name("class")
        .and_then(|ty| extract_type_node(ty, ty.end_byte(), source));
    let on_type_source = parsed_on_type.as_ref().map(|ty| ty.source.clone());

    ParsedExtensionSurface {
        name,
        on_type_source,
        parsed_on_type,
        annotations: direct_annotations(node, source),
        span: text_range(node),
    }
}

/// Extracts an extension type and its representation metadata.
pub(super) fn extract_extension_type(
    node: Node<'_>,
    source: &SourceText,
) -> ParsedExtensionTypeSurface {
    let name = node
        .child_by_field_name("name")
        .map(|name| node_text(name, source))
        .unwrap_or_default();
    let representation = node.child_by_field_name("representation");
    let representation_name = representation
        .and_then(|representation| representation.child_by_field_name("name"))
        .map(|name| node_text(name, source))
        .unwrap_or_default();
    let parsed_representation_type = representation
        .and_then(|representation| representation.child_by_field_name("type"))
        .and_then(|ty| extract_type_node(ty, ty.end_byte(), source));
    let representation_type_source = parsed_representation_type
        .as_ref()
        .map(|ty| ty.source.clone());

    ParsedExtensionTypeSurface {
        name,
        representation_name,
        representation_type_source,
        parsed_representation_type,
        annotations: direct_annotations(node, source),
        span: text_range(node),
    }
}

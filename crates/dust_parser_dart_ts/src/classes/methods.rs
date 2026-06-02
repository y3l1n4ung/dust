use dust_parser_dart::{ParsedAnnotation, ParsedMethodParamSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::{extract_annotation, extract_descendant_annotations},
    syntax::{find_first_descendant, find_last_descendant_text, node_text, text_range},
};

use super::parse_text::{
    determine_parameter_kind, extract_default_value_source, extract_parameter_type,
    trailing_default_value_source,
};

pub(super) fn extract_method(
    node: Node<'_>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Option<dust_parser_dart::ParsedMethodSurface> {
    let signature = find_first_descendant(node, "method_signature")
        .or_else(|| find_first_descendant(node, "function_signature"))
        .or_else(|| find_first_descendant(node, "declaration"))?;

    let name_node = signature.child_by_field_name("name")?;
    let name = node_text(name_node, source);

    let header_text = node_text(signature, source);
    let declaration_text = node_text(node, source);
    let is_static = header_text.contains("static");
    let is_external = header_text.contains("external");

    let return_type_source = signature
        .child_by_field_name("type")
        .map(|t| node_text(t, source))
        .or_else(|| extract_parameter_type(&header_text, &name));

    let params = find_first_descendant(signature, "formal_parameter_list")
        .map(|list| extract_method_params(list, source))
        .unwrap_or_default();

    let params_node = find_first_descendant(signature, "formal_parameter_list");
    let body_source = params_node.and_then(|params| {
        let end_offset = params.end_byte().saturating_sub(node.start_byte());
        let after_params = declaration_text[end_offset.min(declaration_text.len())..].trim();
        (!after_params.is_empty()).then(|| after_params.to_owned())
    });
    let has_body = if let Some(params) = params_node {
        let end_offset = params.end_byte().saturating_sub(node.start_byte());
        let after_params = &declaration_text[end_offset.min(declaration_text.len())..];
        after_params.contains('{') || after_params.contains("=>")
    } else {
        declaration_text.contains('{') || declaration_text.contains("=>")
    };

    Some(dust_parser_dart::ParsedMethodSurface {
        name,
        is_static,
        is_external,
        annotations: annotations.to_vec(),
        return_type_source,
        has_body,
        body_source,
        params,
        span: text_range(signature),
    })
}

fn extract_method_params(node: Node<'_>, source: &SourceText) -> Vec<ParsedMethodParamSurface> {
    let mut params = Vec::new();
    collect_method_formal_parameters(node, source, &mut params, &mut Vec::new());
    params
}

fn collect_method_formal_parameters(
    node: Node<'_>,
    source: &SourceText,
    out: &mut Vec<ParsedMethodParamSurface>,
    pending_annotations: &mut Vec<ParsedAnnotation>,
) {
    if !node.is_named() {
        return;
    }

    if node.kind() == "annotation" {
        pending_annotations.push(extract_annotation(node, source));
        return;
    }

    if node.is_named() && matches!(node.kind(), "formal_parameter" | "default_formal_parameter") {
        let mut param = extract_method_formal_parameter(node, source);
        if !pending_annotations.is_empty() {
            let mut annotations = std::mem::take(pending_annotations);
            annotations.extend(param.annotations);
            param.annotations = annotations;
        }
        out.push(param);
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_method_formal_parameters(child, source, out, pending_annotations);
    }
}

fn extract_method_formal_parameter(
    node: Node<'_>,
    source: &SourceText,
) -> ParsedMethodParamSurface {
    let text = node_text(node, source);
    let name = find_last_descendant_text(node, source, &["identifier"]).unwrap_or_default();
    let type_source = extract_parameter_type(&text, &name);

    ParsedMethodParamSurface {
        name,
        annotations: extract_descendant_annotations(node, source),
        type_source,
        kind: determine_parameter_kind(node, source),
        has_default: text.contains('=') || trailing_default_value_source(node, source).is_some(),
        default_value_source: extract_default_value_source(&text)
            .or_else(|| trailing_default_value_source(node, source)),
        span: text_range(node),
    }
}

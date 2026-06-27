use dust_parser_dart::{ParameterKind, ParsedAnnotation, ParsedMethodParamSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::{extract_annotation, extract_direct_annotations},
    syntax::{
        direct_named_child, find_first_descendant, has_direct_child_kind, node_text, text_range,
    },
    types::extract_type_before,
};

use super::parse_text::{
    default_value_source, is_required_parameter, optional_parameter_kind, parameter_name_node,
};

/// Extracts a parsed method surface from a class member declaration.
pub(super) fn extract_method(
    node: Node<'_>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Option<dust_parser_dart::ParsedMethodSurface> {
    let signature = find_first_descendant(node, "method_signature")
        .or_else(|| find_first_descendant(node, "function_signature"))
        .or_else(|| find_first_descendant(node, "declaration"))?;
    let callable_signature = callable_signature(signature);

    let name_node = callable_signature.child_by_field_name("name")?;
    let name = node_text(name_node, source);

    let is_static =
        has_direct_child_kind(node, "static") || has_direct_child_kind(signature, "static");
    let is_external =
        has_direct_child_kind(node, "external") || has_direct_child_kind(signature, "external");

    let parsed_return_type =
        extract_type_before(callable_signature, name_node.start_byte(), source);
    let return_type_source = parsed_return_type.as_ref().map(|ty| ty.source.clone());

    let params = find_first_descendant(callable_signature, "formal_parameter_list")
        .map(|list| extract_method_params(list, source))
        .unwrap_or_default();

    let body_source = method_body_node(node, signature).map(|body| node_text(body, source));
    let has_body = body_source.is_some();

    Some(dust_parser_dart::ParsedMethodSurface {
        name,
        is_static,
        is_external,
        annotations: annotations.to_vec(),
        return_type_source,
        parsed_return_type,
        has_body,
        body_source,
        params,
        span: text_range(signature),
    })
}

/// Resolves a wrapper node to the callable signature that carries the name.
fn callable_signature(node: Node<'_>) -> Node<'_> {
    if node.kind() != "method_signature" {
        return node;
    }

    direct_named_child(node, "function_signature")
        .or_else(|| direct_named_child(node, "getter_signature"))
        .or_else(|| direct_named_child(node, "setter_signature"))
        .or_else(|| direct_named_child(node, "operator_signature"))
        .unwrap_or(node)
}

/// Finds the method body attached to a declaration or signature wrapper.
fn method_body_node<'tree>(node: Node<'tree>, signature: Node<'tree>) -> Option<Node<'tree>> {
    direct_named_child(node, "function_body").or_else(|| {
        signature
            .parent()
            .and_then(|parent| direct_named_child(parent, "function_body"))
    })
}

/// Extracts method or function parameters from a formal parameter list.
pub(crate) fn extract_method_params(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedMethodParamSurface> {
    let mut params = Vec::new();
    collect_method_formal_parameters(
        node,
        source,
        &mut params,
        &mut Vec::new(),
        ParameterKind::Positional,
    );
    params
}

/// Recursively collects method parameters while preserving leading annotations.
fn collect_method_formal_parameters(
    node: Node<'_>,
    source: &SourceText,
    out: &mut Vec<ParsedMethodParamSurface>,
    pending_annotations: &mut Vec<ParsedAnnotation>,
    kind: ParameterKind,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        match child.kind() {
            "annotation" => pending_annotations.push(extract_annotation(child, source)),
            "formal_parameter" | "default_formal_parameter" => {
                push_method_formal_parameter(child, source, out, pending_annotations, kind);
            }
            "optional_formal_parameters" => {
                collect_method_optional_formal_parameters(child, source, out, pending_annotations);
            }
            _ => {}
        }
    }
}

/// Collects parameters inside optional positional or named method groups.
fn collect_method_optional_formal_parameters(
    node: Node<'_>,
    source: &SourceText,
    out: &mut Vec<ParsedMethodParamSurface>,
    pending_annotations: &mut Vec<ParsedAnnotation>,
) {
    let kind = optional_parameter_kind(node, source);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        match child.kind() {
            "annotation" => pending_annotations.push(extract_annotation(child, source)),
            "formal_parameter" | "default_formal_parameter" => {
                push_method_formal_parameter(child, source, out, pending_annotations, kind);
            }
            _ => {}
        }
    }
}

/// Adds one parsed method parameter and applies any pending annotations.
fn push_method_formal_parameter(
    node: Node<'_>,
    source: &SourceText,
    out: &mut Vec<ParsedMethodParamSurface>,
    pending_annotations: &mut Vec<ParsedAnnotation>,
    kind: ParameterKind,
) {
    let mut param = extract_method_formal_parameter(node, source, kind);
    if !pending_annotations.is_empty() {
        let mut annotations = std::mem::take(pending_annotations);
        annotations.extend(param.annotations);
        param.annotations = annotations;
    }
    out.push(param);
}

/// Converts one tree-sitter parameter node into method parameter metadata.
fn extract_method_formal_parameter(
    node: Node<'_>,
    source: &SourceText,
    kind: ParameterKind,
) -> ParsedMethodParamSurface {
    let name_node = parameter_name_node(node);
    let name = name_node
        .map(|node| node_text(node, source))
        .unwrap_or_default();
    let parsed_type =
        name_node.and_then(|name| extract_type_before(node, name.start_byte(), source));
    let type_source = parsed_type.as_ref().map(|ty| ty.source.clone());
    let default_value_source = default_value_source(node, source);

    ParsedMethodParamSurface {
        name,
        annotations: extract_direct_annotations(node, source),
        type_source,
        parsed_type,
        kind,
        is_required: is_required_parameter(node, source),
        has_default: default_value_source.is_some(),
        default_value_source,
        span: text_range(node),
    }
}

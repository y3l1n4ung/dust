use dust_parser_dart::{ParsedConstructorParamSurface, ParsedConstructorSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{
    find_first_descendant, find_first_descendant_by, find_last_descendant_text, node_text,
    text_range,
};

use super::parse_text::{
    determine_parameter_kind, extract_default_value_source, extract_parameter_type,
    extract_redirect_target, extract_redirect_target_name, trailing_default_value_source,
};

pub(super) fn extract_constructor(node: Node<'_>, source: &SourceText) -> ParsedConstructorSurface {
    let Some(signature) = find_first_descendant_by(node, |candidate| {
        matches!(
            candidate.kind(),
            "constant_constructor_signature"
                | "constructor_signature"
                | "factory_constructor_signature"
                | "redirecting_factory_constructor_signature"
        )
    }) else {
        return ParsedConstructorSurface {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            params: Vec::new(),
            span: text_range(node),
        };
    };
    let declaration_text = node_text(node, source);
    let is_factory = declaration_text
        .split_whitespace()
        .any(|word| word == "factory");

    let mut identifiers = Vec::new();
    let mut cursor = signature.walk();
    for child in signature
        .children(&mut cursor)
        .filter(|child| child.is_named())
    {
        if child.kind() == "identifier" {
            identifiers.push(node_text(child, source));
        }
    }

    let name = if identifiers.len() > 1 {
        identifiers.get(1).cloned()
    } else {
        None
    };

    let params = find_first_descendant(signature, "formal_parameter_list")
        .map(|list| extract_constructor_params(list, source))
        .unwrap_or_default();
    let redirected_target_source = (signature.kind()
        == "redirecting_factory_constructor_signature")
        .then(|| extract_redirect_target(&declaration_text))
        .flatten();
    let redirected_target_name = redirected_target_source
        .as_deref()
        .and_then(extract_redirect_target_name);

    ParsedConstructorSurface {
        name,
        is_factory,
        redirected_target_source,
        redirected_target_name,
        params,
        span: text_range(signature),
    }
}

fn extract_constructor_params(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedConstructorParamSurface> {
    let mut params = Vec::new();
    collect_formal_parameters(node, source, &mut params);
    params
}

fn collect_formal_parameters(
    node: Node<'_>,
    source: &SourceText,
    out: &mut Vec<ParsedConstructorParamSurface>,
) {
    if node.is_named() && matches!(node.kind(), "formal_parameter" | "default_formal_parameter") {
        out.push(extract_formal_parameter(node, source));
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_formal_parameters(child, source, out);
    }
}

fn extract_formal_parameter(node: Node<'_>, source: &SourceText) -> ParsedConstructorParamSurface {
    let text = node_text(node, source);
    let name = find_last_descendant_text(node, source, &["identifier"]).unwrap_or_default();
    let type_source = extract_parameter_type(&text, &name);

    ParsedConstructorParamSurface {
        name,
        type_source,
        kind: determine_parameter_kind(node, source),
        has_default: text.contains('=') || trailing_default_value_source(node, source).is_some(),
        default_value_source: extract_default_value_source(&text)
            .or_else(|| trailing_default_value_source(node, source)),
        span: text_range(node),
    }
}

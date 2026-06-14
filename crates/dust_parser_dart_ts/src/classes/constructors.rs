use dust_parser_dart::{ParsedConstructorParamSurface, ParsedConstructorSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{find_first_descendant, find_first_descendant_by, node_text, text_range};
use crate::types::extract_type_before;

use super::parse_text::{default_value_source, determine_parameter_kind, parameter_name_node};

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
    let is_factory = matches!(
        signature.kind(),
        "factory_constructor_signature" | "redirecting_factory_constructor_signature"
    );
    let name = constructor_name(signature, source);

    let params = find_first_descendant(signature, "formal_parameter_list")
        .map(|list| extract_constructor_params(list, source))
        .unwrap_or_default();
    let (redirected_target_source, redirected_target_name) =
        redirecting_factory_target(signature, source);

    ParsedConstructorSurface {
        name,
        is_factory,
        redirected_target_source,
        redirected_target_name,
        params,
        span: text_range(signature),
    }
}

fn constructor_name(signature: Node<'_>, source: &SourceText) -> Option<String> {
    let mut cursor = signature.walk();
    let identifiers = signature
        .children_by_field_name("name", &mut cursor)
        .filter(|child| child.is_named() && child.kind() == "identifier")
        .map(|identifier| node_text(identifier, source))
        .collect::<Vec<_>>();
    identifiers.get(1).cloned()
}

fn redirecting_factory_target(
    signature: Node<'_>,
    source: &SourceText,
) -> (Option<String>, Option<String>) {
    if signature.kind() != "redirecting_factory_constructor_signature" {
        return (None, None);
    }

    let mut cursor = signature.walk();
    let target_nodes = signature
        .children_by_field_name("target", &mut cursor)
        .collect::<Vec<_>>();
    let Some(first_target) = target_nodes.first() else {
        return (None, None);
    };
    let Some(last_target) = target_nodes.last() else {
        return (None, None);
    };

    let target_end = signature
        .child_by_field_name("target_constructor")
        .map(|constructor| constructor.end_byte())
        .unwrap_or_else(|| last_target.end_byte());
    let target_source = source
        .as_str()
        .get(first_target.start_byte()..target_end)
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .map(ToOwned::to_owned);

    let target_name = target_source.clone();
    (target_source, target_name)
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
    let name_node = parameter_name_node(node);
    let name = name_node
        .map(|node| node_text(node, source))
        .unwrap_or_default();
    let parsed_type =
        name_node.and_then(|name| extract_type_before(node, name.start_byte(), source));
    let type_source = parsed_type.as_ref().map(|ty| ty.source.clone());
    let default_value_source = default_value_source(node, source);

    ParsedConstructorParamSurface {
        name,
        type_source,
        parsed_type,
        kind: determine_parameter_kind(node, source),
        has_default: default_value_source.is_some(),
        default_value_source,
        span: text_range(node),
    }
}

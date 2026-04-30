use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedClassKind, ParsedClassSurface,
    ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedFieldSurface,
};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::{extract_annotation, extract_member_annotations},
    syntax::{
        class_header_text, find_first_descendant, find_first_descendant_by,
        find_first_descendant_text, find_last_descendant_text, first_non_annotation_named_child,
        has_descendant_kind, node_text, text_range,
    },
};

pub(crate) fn extract_classes(root: Node<'_>, source: &SourceText) -> Vec<ParsedClassSurface> {
    let mut classes = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        if child.kind() == "class_declaration" {
            classes.push(extract_class(child, source));
        }
    }

    classes
}

fn extract_class(node: Node<'_>, source: &SourceText) -> ParsedClassSurface {
    let header = class_header_text(node, source);
    let kind = if header.contains("mixin class") {
        ParsedClassKind::MixinClass
    } else {
        ParsedClassKind::Class
    };
    let is_abstract = header.split_whitespace().any(|word| word == "abstract");
    let class_name = node
        .child_by_field_name("name")
        .map(|name| node_text(name, source))
        .unwrap_or_default();
    let superclass_name = node
        .child_by_field_name("superclass")
        .and_then(|superclass| {
            find_first_descendant_text(superclass, source, &["type_identifier"])
        });
    let mut annotations = Vec::new();
    let mut fields = Vec::new();
    let mut constructors = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }

    if let Some(body) = node.child_by_field_name("body") {
        let mut body_cursor = body.walk();
        for member in body
            .children(&mut body_cursor)
            .filter(|child| child.is_named() && child.kind() == "class_member")
        {
            let member_annotations = extract_member_annotations(member, source);
            if let Some(declaration) = first_non_annotation_named_child(member) {
                if has_descendant_kind(declaration, "constant_constructor_signature")
                    || has_descendant_kind(declaration, "constructor_signature")
                {
                    constructors.push(extract_constructor(declaration, source));
                } else if has_descendant_kind(declaration, "initialized_identifier_list") {
                    fields.extend(extract_fields(declaration, &member_annotations, source));
                }
            }
        }
    }

    ParsedClassSurface {
        kind,
        name: class_name,
        is_abstract,
        superclass_name,
        annotations,
        fields,
        constructors,
        span: text_range(node),
    }
}

fn extract_fields(
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

fn extract_constructor(node: Node<'_>, source: &SourceText) -> ParsedConstructorSurface {
    let Some(signature) = find_first_descendant_by(node, |candidate| {
        matches!(
            candidate.kind(),
            "constant_constructor_signature" | "constructor_signature"
        )
    }) else {
        return ParsedConstructorSurface {
            name: None,
            params: Vec::new(),
            span: text_range(node),
        };
    };

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

    ParsedConstructorSurface {
        name,
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
    if node.is_named() && node.kind() == "formal_parameter" {
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
        has_default: text.contains('='),
        span: text_range(node),
    }
}

fn determine_parameter_kind(node: Node<'_>, source: &SourceText) -> ParameterKind {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "optional_formal_parameters" {
            let text = node_text(parent, source);
            return if text.trim_start().starts_with('{') {
                ParameterKind::Named
            } else {
                ParameterKind::Positional
            };
        }
        current = parent.parent();
    }

    ParameterKind::Positional
}

fn extract_type_prefix(declaration_text: &str, type_end: usize) -> Option<String> {
    let prefix = declaration_text.get(..type_end)?.trim();
    let stripped = strip_prefix_modifiers(prefix);
    if stripped.is_empty() || stripped == "var" {
        None
    } else {
        Some(stripped.to_owned())
    }
}

fn extract_parameter_type(text: &str, name: &str) -> Option<String> {
    if name.is_empty() || text.contains("this.") || text.contains("super.") {
        return None;
    }

    let end = text.rfind(name)?;
    let prefix = text.get(..end)?.trim();
    let stripped = strip_prefix_modifiers(prefix);
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_owned())
    }
}

fn strip_prefix_modifiers(text: &str) -> &str {
    let mut remaining = text.trim();
    let modifiers = [
        "external",
        "static",
        "covariant",
        "late",
        "final",
        "const",
        "required",
        "factory",
        "var",
    ];

    loop {
        let mut matched = false;
        for modifier in modifiers {
            if let Some(rest) = remaining.strip_prefix(modifier) {
                remaining = rest.trim_start();
                matched = true;
                break;
            }
        }

        if !matched {
            return remaining.trim();
        }
    }
}

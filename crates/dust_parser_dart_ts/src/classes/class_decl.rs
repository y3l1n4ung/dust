use dust_parser_dart::{ParsedClassKind, ParsedClassSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::{extract_annotation, extract_member_annotations},
    syntax::{
        class_header_text, find_first_descendant_text, first_non_annotation_named_child,
        has_descendant_kind, node_text, text_range,
    },
};

use super::{constructors::extract_constructor, fields::extract_fields, methods::extract_method};

pub(super) fn extract_class(node: Node<'_>, source: &SourceText) -> ParsedClassSurface {
    let header = class_header_text(node, source);
    let kind = if header.contains("mixin class") {
        ParsedClassKind::MixinClass
    } else {
        ParsedClassKind::Class
    };
    let is_abstract = header.split_whitespace().any(|word| word == "abstract");
    let is_interface = header.contains("interface class");
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
    let mut methods = Vec::new();

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
                    || has_descendant_kind(declaration, "factory_constructor_signature")
                    || has_descendant_kind(declaration, "redirecting_factory_constructor_signature")
                {
                    constructors.push(extract_constructor(declaration, source));
                } else if has_descendant_kind(declaration, "initialized_identifier_list") {
                    fields.extend(extract_fields(declaration, &member_annotations, source));
                } else if has_descendant_kind(declaration, "declaration")
                    || has_descendant_kind(declaration, "method_signature")
                    || has_descendant_kind(declaration, "function_signature")
                {
                    // Tree-sitter-dart uses different kinds for methods depending on if they have a body.
                    // We'll try to extract them if they look like methods.
                    if let Some(method) = extract_method(declaration, &member_annotations, source) {
                        methods.push(method);
                    }
                }
            }
        }
    }

    ParsedClassSurface {
        kind,
        name: class_name,
        is_abstract,
        is_interface,
        superclass_name,
        annotations,
        fields,
        constructors,
        methods,
        span: text_range(node),
    }
}

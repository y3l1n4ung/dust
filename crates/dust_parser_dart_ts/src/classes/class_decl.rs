use dust_parser_dart::{ParsedClassKind, ParsedClassSurface};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::{extract_annotation, extract_member_annotations},
    syntax::{
        find_first_descendant_text, first_non_annotation_named_child, has_direct_child_kind,
        node_text, text_range,
    },
};

use super::{
    constructors::{extract_constructor, is_constructor_signature_kind},
    fields::extract_fields_from_identifier_list,
    methods::extract_method,
};

/// Extracts a Dust class surface from one tree-sitter class declaration.
pub(super) fn extract_class(node: Node<'_>, source: &SourceText) -> ParsedClassSurface {
    let kind = if has_direct_child_kind(node, "mixin") {
        ParsedClassKind::MixinClass
    } else {
        ParsedClassKind::Class
    };
    let is_abstract = has_direct_child_kind(node, "abstract");
    let is_interface = has_direct_child_kind(node, "interface");
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
                match classify_class_member(declaration) {
                    Some(ClassMemberShape::Constructor(signature)) => {
                        constructors.push(extract_constructor(signature, source));
                    }
                    Some(ClassMemberShape::Field(identifier_list)) => {
                        fields.extend(extract_fields_from_identifier_list(
                            declaration,
                            identifier_list,
                            &member_annotations,
                            source,
                        ));
                    }
                    Some(ClassMemberShape::Method) => {
                        if let Some(method) =
                            extract_method(declaration, &member_annotations, source)
                        {
                            methods.push(method);
                        }
                    }
                    None => {}
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

/// Shape of one class member declaration.
enum ClassMemberShape<'tree> {
    /// Constructor signature node.
    Constructor(Node<'tree>),
    /// Initialized identifier list for fields.
    Field(Node<'tree>),
    /// Method-like member.
    Method,
}

/// Classifies a class member into a shape Dust extracts.
fn classify_class_member<'tree>(node: Node<'tree>) -> Option<ClassMemberShape<'tree>> {
    if is_constructor_signature_kind(node.kind()) {
        return Some(ClassMemberShape::Constructor(node));
    }
    if node.kind() == "initialized_identifier_list" {
        return Some(ClassMemberShape::Field(node));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if let Some(shape) = classify_class_member(child) {
            return Some(shape);
        }
    }

    matches!(
        node.kind(),
        "declaration" | "method_signature" | "function_signature"
    )
    .then_some(ClassMemberShape::Method)
}

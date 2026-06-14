use dust_parser_dart::{
    ParsedAnnotation, ParsedExtensionSurface, ParsedExtensionTypeSurface, ParsedFunctionSurface,
    ParsedMixinSurface, ParsedTopLevelVariableSurface, ParsedTypedefSurface,
};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::{extract_annotation, extract_member_annotations},
    classes::{fields::extract_fields, methods::extract_method_params},
    syntax::{
        direct_named_child, find_first_descendant, has_descendant_kind, node_text, text_range,
    },
    types::{extract_type_before, extract_type_node},
};

pub(crate) struct ParsedTopLevelDeclarations {
    pub(crate) mixins: Vec<ParsedMixinSurface>,
    pub(crate) extensions: Vec<ParsedExtensionSurface>,
    pub(crate) extension_types: Vec<ParsedExtensionTypeSurface>,
    pub(crate) functions: Vec<ParsedFunctionSurface>,
    pub(crate) variables: Vec<ParsedTopLevelVariableSurface>,
    pub(crate) typedefs: Vec<ParsedTypedefSurface>,
}

impl ParsedTopLevelDeclarations {
    pub(crate) fn empty() -> Self {
        Self {
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
        }
    }
}

pub(crate) fn extract_top_level_declarations(
    root: Node<'_>,
    source: &SourceText,
) -> ParsedTopLevelDeclarations {
    let mut declarations = ParsedTopLevelDeclarations::empty();
    let mut pending_annotations = Vec::new();
    let mut current_type = None;

    let mut cursor = root.walk();
    let children = root.children(&mut cursor).collect::<Vec<_>>();
    for child in children {
        if child.kind() == ";" {
            pending_annotations.clear();
            current_type = None;
            continue;
        }

        if !child.is_named() {
            continue;
        }

        match child.kind() {
            "annotation" => pending_annotations.push(extract_annotation(child, source)),
            "mixin_declaration" => {
                declarations.mixins.push(extract_mixin(child, source));
                pending_annotations.clear();
                current_type = None;
            }
            "extension_declaration" => {
                declarations
                    .extensions
                    .push(extract_extension(child, source));
                pending_annotations.clear();
                current_type = None;
            }
            "extension_type_declaration" => {
                declarations
                    .extension_types
                    .push(extract_extension_type(child, source));
                pending_annotations.clear();
                current_type = None;
            }
            "function_signature" => {
                declarations.functions.push(extract_function(
                    child,
                    std::mem::take(&mut pending_annotations),
                    source,
                ));
                current_type = None;
            }
            "initialized_identifier_list" => {
                declarations.variables.extend(extract_initialized_variables(
                    child,
                    current_type,
                    &pending_annotations,
                    source,
                ));
                pending_annotations.clear();
                current_type = None;
            }
            "static_final_declaration_list" => {
                declarations
                    .variables
                    .extend(extract_static_final_variables(
                        child,
                        current_type,
                        &pending_annotations,
                        source,
                    ));
                pending_annotations.clear();
                current_type = None;
            }
            "identifier_list" => {
                declarations.variables.extend(extract_external_variables(
                    child,
                    current_type,
                    &pending_annotations,
                    source,
                ));
                pending_annotations.clear();
                current_type = None;
            }
            "type_alias" => {
                declarations.typedefs.push(extract_typedef(child, source));
                pending_annotations.clear();
                current_type = None;
            }
            kind if is_type_node_kind(kind) => current_type = Some(child),
            "function_body" => current_type = None,
            _ => {}
        }
    }

    declarations
}

fn extract_mixin(node: Node<'_>, source: &SourceText) -> ParsedMixinSurface {
    let name = node
        .child_by_field_name("name")
        .map(|name| node_text(name, source))
        .unwrap_or_default();

    let mut fields = Vec::new();
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for member in body
            .children(&mut cursor)
            .filter(|child| child.is_named() && child.kind() == "class_member")
        {
            let member_annotations = extract_member_annotations(member, source);
            if let Some(declaration) = direct_named_child(member, "declaration") {
                if has_descendant_kind(declaration, "initialized_identifier_list") {
                    fields.extend(extract_fields(declaration, &member_annotations, source));
                }
            }
        }
    }

    ParsedMixinSurface {
        name,
        annotations: direct_annotations(node, source),
        fields,
        span: text_range(node),
    }
}

fn extract_extension(node: Node<'_>, source: &SourceText) -> ParsedExtensionSurface {
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

fn extract_extension_type(node: Node<'_>, source: &SourceText) -> ParsedExtensionTypeSurface {
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

fn extract_function(
    signature: Node<'_>,
    annotations: Vec<ParsedAnnotation>,
    source: &SourceText,
) -> ParsedFunctionSurface {
    let name_node = signature.child_by_field_name("name");
    let name = name_node
        .map(|name| node_text(name, source))
        .unwrap_or_default();
    let parsed_return_type =
        name_node.and_then(|name| extract_type_before(signature, name.start_byte(), source));
    let return_type_source = parsed_return_type.as_ref().map(|ty| ty.source.clone());
    let params = find_first_descendant(signature, "formal_parameter_list")
        .map(|params| extract_method_params(params, source))
        .unwrap_or_default();

    ParsedFunctionSurface {
        name,
        return_type_source,
        parsed_return_type,
        params,
        annotations,
        span: text_range(signature),
    }
}

fn extract_initialized_variables(
    list: Node<'_>,
    type_node: Option<Node<'_>>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Vec<ParsedTopLevelVariableSurface> {
    let parsed_type = type_node.and_then(|ty| extract_type_node(ty, list.start_byte(), source));
    let type_source = parsed_type.as_ref().map(|ty| ty.source.clone());
    let mut variables = Vec::new();
    let mut cursor = list.walk();
    for initialized in list
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "initialized_identifier")
    {
        let name = initialized
            .child_by_field_name("name")
            .map(|name| node_text(name, source))
            .unwrap_or_default();
        let initializer_source = initialized
            .child_by_field_name("value")
            .map(|value| node_text(value, source));
        let initializer_span = initialized.child_by_field_name("value").map(text_range);
        variables.push(ParsedTopLevelVariableSurface {
            name,
            type_source: type_source.clone(),
            parsed_type: parsed_type.clone(),
            initializer_source,
            initializer_span,
            annotations: annotations.to_vec(),
            span: text_range(initialized),
        });
    }
    variables
}

fn extract_static_final_variables(
    list: Node<'_>,
    type_node: Option<Node<'_>>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Vec<ParsedTopLevelVariableSurface> {
    let parsed_type = type_node.and_then(|ty| extract_type_node(ty, list.start_byte(), source));
    let type_source = parsed_type.as_ref().map(|ty| ty.source.clone());
    let mut variables = Vec::new();
    let mut cursor = list.walk();
    for declaration in list
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "static_final_declaration")
    {
        let name = declaration
            .child_by_field_name("name")
            .map(|name| node_text(name, source))
            .unwrap_or_default();
        let initializer_source = declaration
            .child_by_field_name("value")
            .map(|value| node_text(value, source));
        let initializer_span = declaration.child_by_field_name("value").map(text_range);
        variables.push(ParsedTopLevelVariableSurface {
            name,
            type_source: type_source.clone(),
            parsed_type: parsed_type.clone(),
            initializer_source,
            initializer_span,
            annotations: annotations.to_vec(),
            span: text_range(declaration),
        });
    }
    variables
}

fn extract_external_variables(
    list: Node<'_>,
    type_node: Option<Node<'_>>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Vec<ParsedTopLevelVariableSurface> {
    let parsed_type = type_node.and_then(|ty| extract_type_node(ty, list.start_byte(), source));
    let type_source = parsed_type.as_ref().map(|ty| ty.source.clone());
    let mut variables = Vec::new();
    let mut cursor = list.walk();
    for identifier in list
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "identifier")
    {
        variables.push(ParsedTopLevelVariableSurface {
            name: node_text(identifier, source),
            type_source: type_source.clone(),
            parsed_type: parsed_type.clone(),
            initializer_source: None,
            initializer_span: None,
            annotations: annotations.to_vec(),
            span: text_range(identifier),
        });
    }
    variables
}

fn extract_typedef(node: Node<'_>, source: &SourceText) -> ParsedTypedefSurface {
    let equals = direct_child(node, "=");
    let name_node = if let Some(equals) = equals {
        first_type_identifier_before(node, equals.start_byte())
    } else {
        direct_named_child_before(node, "formal_parameter_list")
            .and_then(|params| previous_type_identifier(node, params.start_byte()))
    };
    let name = name_node
        .map(|name| node_text(name, source))
        .unwrap_or_default();
    let alias_type_node = equals.and_then(|equals| first_type_node_after(node, equals.end_byte()));
    let alias_end = alias_type_node
        .and_then(|_| direct_child(node, ";"))
        .map(|semicolon| semicolon.start_byte())
        .unwrap_or_else(|| node.end_byte());
    let parsed_aliased_type =
        alias_type_node.and_then(|ty| extract_type_node(ty, alias_end, source));
    let aliased_type_source = parsed_aliased_type.as_ref().map(|ty| ty.source.clone());

    ParsedTypedefSurface {
        name,
        aliased_type_source,
        parsed_aliased_type,
        annotations: direct_annotations(node, source),
        span: text_range(node),
    }
}

fn direct_annotations(node: Node<'_>, source: &SourceText) -> Vec<ParsedAnnotation> {
    let mut annotations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }
    annotations
}

fn direct_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn direct_named_child_before<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.is_named() && child.kind() == kind)
}

fn first_type_identifier_before<'tree>(
    node: Node<'tree>,
    boundary_byte: usize,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| {
            child.is_named()
                && child.kind() == "type_identifier"
                && child.start_byte() < boundary_byte
        })
        .last()
}

fn previous_type_identifier<'tree>(node: Node<'tree>, boundary_byte: usize) -> Option<Node<'tree>> {
    first_type_identifier_before(node, boundary_byte)
}

fn first_type_node_after<'tree>(node: Node<'tree>, boundary_byte: usize) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|child| {
        child.is_named() && is_type_node_kind(child.kind()) && child.start_byte() >= boundary_byte
    })
}

fn is_type_node_kind(kind: &str) -> bool {
    matches!(
        kind,
        "type_identifier" | "void_type" | "function_type" | "record_type"
    )
}

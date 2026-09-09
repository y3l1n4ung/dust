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

/// Walking a declaration's tree-sitter node for annotations, names and types.
mod nodes;
use self::nodes::*;

/// Extracting top-level variable declarations in their three Dart spellings.
mod variables;
use self::variables::*;

/// Extracting extension and extension type declarations.
mod extensions;
use self::extensions::*;

/// Parsed top-level declarations not covered by class and enum extraction.
pub(crate) struct ParsedTopLevelDeclarations {
    /// Parsed mixin declarations.
    pub(crate) mixins: Vec<ParsedMixinSurface>,
    /// Parsed extension declarations.
    pub(crate) extensions: Vec<ParsedExtensionSurface>,
    /// Parsed extension type declarations.
    pub(crate) extension_types: Vec<ParsedExtensionTypeSurface>,
    /// Parsed top-level function declarations.
    pub(crate) functions: Vec<ParsedFunctionSurface>,
    /// Parsed top-level variable declarations.
    pub(crate) variables: Vec<ParsedTopLevelVariableSurface>,
    /// Parsed typedef declarations.
    pub(crate) typedefs: Vec<ParsedTypedefSurface>,
}

impl ParsedTopLevelDeclarations {
    /// Creates an empty top-level declaration set.
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

/// Extracts all supported top-level declarations from a compilation unit root.
pub(crate) fn extract_top_level_declarations(
    root: Node<'_>,
    source: &SourceText,
) -> ParsedTopLevelDeclarations {
    let mut declarations = ParsedTopLevelDeclarations::empty();
    let mut pending_annotations = Vec::new();
    let mut current_type = None;

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
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

/// Returns whether the root contains declarations this module can extract.
pub(crate) fn has_extractable_top_level_declarations(root: Node<'_>) -> bool {
    let mut cursor = root.walk();
    root.children(&mut cursor)
        .filter(|child| child.is_named())
        .any(|child| {
            matches!(
                child.kind(),
                "mixin_declaration"
                    | "extension_declaration"
                    | "extension_type_declaration"
                    | "function_signature"
                    | "initialized_identifier_list"
                    | "static_final_declaration_list"
                    | "identifier_list"
                    | "type_alias"
            )
        })
}

/// Extracts a mixin surface and its fields from a mixin declaration.
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

/// Extracts a top-level function surface from a function signature.
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

/// Extracts a typedef declaration and any aliased type source.
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

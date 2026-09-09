//! Lowering for top-level declarations: mixins, extensions, functions,
//! variables and typedefs.

use super::*;

/// Lowers parsed mixins and their unresolved fields.
pub(super) fn lower_mixins(
    file_id: dust_text::FileId,
    mixins: &[ParsedMixinSurface],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<MixinIr> {
    mixins
        .iter()
        .map(|mixin| MixinIr {
            name: lower_name_ir(file_id, &mixin.name, mixin.span),
            annotations: mixin
                .annotations
                .iter()
                .map(|annotation| lower_annotation_ir(file_id, annotation, catalog))
                .collect(),
            fields: mixin
                .fields
                .iter()
                .map(|field| lower_unresolved_field(file_id, field, diagnostics))
                .collect(),
            span: SpanIr::new(file_id, mixin.span),
        })
        .collect()
}

/// Lowers parsed extensions and their `on` type.
pub(super) fn lower_extensions(
    file_id: dust_text::FileId,
    extensions: &[ParsedExtensionSurface],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ExtensionIr> {
    extensions
        .iter()
        .map(|extension| {
            let on_type = lower_type(
                extension.parsed_on_type.as_ref(),
                extension.on_type_source.as_deref(),
            );
            diagnostics.extend(on_type.diagnostics);

            ExtensionIr {
                name: extension
                    .name
                    .as_deref()
                    .map(|name| lower_name_ir(file_id, name, extension.span)),
                on_type: on_type.value,
                annotations: extension
                    .annotations
                    .iter()
                    .map(|annotation| lower_annotation_ir(file_id, annotation, catalog))
                    .collect(),
                span: SpanIr::new(file_id, extension.span),
            }
        })
        .collect()
}

/// Lowers parsed extension types and their representation field.
pub(super) fn lower_extension_types(
    file_id: dust_text::FileId,
    extension_types: &[ParsedExtensionTypeSurface],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ExtensionTypeIr> {
    extension_types
        .iter()
        .map(|extension_type| {
            let representation_type = lower_type(
                extension_type.parsed_representation_type.as_ref(),
                extension_type.representation_type_source.as_deref(),
            );
            diagnostics.extend(representation_type.diagnostics);

            ExtensionTypeIr {
                name: lower_name_ir(file_id, &extension_type.name, extension_type.span),
                annotations: extension_type
                    .annotations
                    .iter()
                    .map(|annotation| lower_annotation_ir(file_id, annotation, catalog))
                    .collect(),
                representation: FieldIr {
                    name: extension_type.representation_name.clone(),
                    ty: representation_type.value,
                    span: SpanIr::new(file_id, extension_type.span),
                    has_default: false,
                    serde: None,
                    configs: Vec::new(),
                },
                span: SpanIr::new(file_id, extension_type.span),
            }
        })
        .collect()
}

/// Lowers parsed top-level functions and their parameters.
pub(super) fn lower_functions(
    file_id: dust_text::FileId,
    functions: &[ParsedFunctionSurface],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<FunctionIr> {
    functions
        .iter()
        .map(|function| {
            let return_type = lower_type(
                function.parsed_return_type.as_ref(),
                function.return_type_source.as_deref(),
            );
            diagnostics.extend(return_type.diagnostics);

            FunctionIr {
                name: lower_name_ir(file_id, &function.name, function.span),
                return_type: return_type.value,
                params: lower_unresolved_method_params(file_id, &function.params, diagnostics),
                annotations: function
                    .annotations
                    .iter()
                    .map(|annotation| lower_annotation_ir(file_id, annotation, catalog))
                    .collect(),
                span: SpanIr::new(file_id, function.span),
            }
        })
        .collect()
}

/// Lowers parsed top-level variables and initializers.
pub(super) fn lower_variables(
    file_id: dust_text::FileId,
    variables: &[ParsedTopLevelVariableSurface],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<TopLevelVariableIr> {
    variables
        .iter()
        .map(|variable| {
            let ty = lower_type(
                variable.parsed_type.as_ref(),
                variable.type_source.as_deref(),
            );
            diagnostics.extend(ty.diagnostics);

            TopLevelVariableIr {
                name: lower_name_ir(file_id, &variable.name, variable.span),
                ty: ty.value,
                initializer: variable
                    .initializer_source
                    .as_ref()
                    .map(|source| ExprSourceIr {
                        source: source.clone(),
                        span: SpanIr::new(
                            file_id,
                            variable.initializer_span.unwrap_or(variable.span),
                        ),
                    }),
                annotations: variable
                    .annotations
                    .iter()
                    .map(|annotation| lower_annotation_ir(file_id, annotation, catalog))
                    .collect(),
                span: SpanIr::new(file_id, variable.span),
            }
        })
        .collect()
}

/// Lowers parsed typedefs and aliased type sources.
pub(super) fn lower_typedefs(
    file_id: dust_text::FileId,
    typedefs: &[ParsedTypedefSurface],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<TypedefIr> {
    typedefs
        .iter()
        .map(|typedef| {
            let aliased_type = lower_type(
                typedef.parsed_aliased_type.as_ref(),
                typedef.aliased_type_source.as_deref(),
            );
            diagnostics.extend(aliased_type.diagnostics);

            TypedefIr {
                name: lower_name_ir(file_id, &typedef.name, typedef.span),
                aliased_type: aliased_type.value,
                annotations: typedef
                    .annotations
                    .iter()
                    .map(|annotation| lower_annotation_ir(file_id, annotation, catalog))
                    .collect(),
                span: SpanIr::new(file_id, typedef.span),
            }
        })
        .collect()
}

/// Lowers one resolved enum into semantic IR.
pub(super) fn lower_enum(e: &mut dust_resolver::ResolvedEnum) -> LoweringOutcome<EnumIr> {
    let diagnostics: Vec<Diagnostic> = Vec::new();
    let serde = e.serde.take();
    let variants: Vec<EnumVariantIr> = e
        .variants
        .iter_mut()
        .map(|v| EnumVariantIr {
            name: v.name.clone(),
            serde: v.serde.take(),
            span: v.span,
        })
        .collect();
    LoweringOutcome {
        value: EnumIr {
            name: e.name.clone(),
            span: e.span,
            variants,
            traits: e.traits.clone(),
            serde,
        },
        diagnostics,
    }
}

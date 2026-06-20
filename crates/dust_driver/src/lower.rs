/// Inherited field and constructor parameter lowering helpers.
mod inheritance;
/// Text parsing helpers used by lower-level Dart source parsing.
mod parse_support;
#[path = "lower/query_calls.rs"]
/// SQL query call lowering.
mod query_calls;
/// SerDe configuration lowering.
mod serde;
/// SerDe annotation argument parsing.
mod serde_parse;
mod tests_declarations;
mod tests_directives;
mod tests_inheritance;
mod tests_serde;
mod tests_type;
/// Dart type lowering and fallback parsing.
mod type_parse;

use std::collections::{HashMap, HashSet};

use dust_diagnostics::Diagnostic;
use dust_ir::{
    AnnotationIr, ClassIr, ConstructorIr, ConstructorParamIr, DartFileIr, EnumIr, EnumVariantIr,
    ExportIr, ExprSourceIr, ExtensionIr, ExtensionTypeIr, FieldIr, FunctionIr, ImportIr,
    LibraryDeclIr, LoweringOutcome, MethodIr, MethodParamIr, MixinIr, NameIr, ParamKind, PartIr,
    PartOfIr, SpanIr, TopLevelVariableIr, TypedefIr,
};
use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedDirective, ParsedFieldSurface, ParsedMethodParamSurface,
};
use dust_resolver::{ResolvedClass, ResolvedLibrary};

use self::{
    inheritance::{infer_param_type, merged_fields_for_class, resolve_constructor_param_types},
    query_calls::lower_query_calls,
    serde::{lower_class_serde_config, lower_field_serde_config},
    type_parse::lower_type,
};

/// Lowers one resolved library into semantic IR.
pub(crate) fn lower_library(library: &ResolvedLibrary) -> LoweringOutcome<DartFileIr> {
    let mut diagnostics = Vec::new();
    let required_classes = lowering_required_class_names(library);
    let mut classes = library
        .classes
        .iter()
        .filter(|class| required_classes.contains(class.name.as_str()))
        .map(|class| {
            let outcome = lower_class(class);
            diagnostics.extend(outcome.diagnostics);
            outcome.value
        })
        .collect::<Vec<_>>();
    let enums = library
        .enums
        .iter()
        .map(|e| {
            let outcome = lower_enum(e);
            diagnostics.extend(outcome.diagnostics);
            outcome.value
        })
        .collect();

    let index_by_name = classes
        .iter()
        .enumerate()
        .map(|(index, class)| (class.name.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut merged_cache = HashMap::new();
    let mut active_stack = Vec::new();
    for index in 0..classes.len() {
        let merged_fields = merged_fields_for_class(
            index,
            &classes,
            &index_by_name,
            &mut merged_cache,
            &mut active_stack,
            &mut diagnostics,
        );
        classes[index].fields = merged_fields;
        resolve_constructor_param_types(&mut classes[index], &mut diagnostics);
    }

    LoweringOutcome {
        value: DartFileIr {
            package_root: String::new(),
            package_name: String::new(),
            source_path: library.source_path.clone(),
            output_path: library.output_path.clone(),
            imports: library_imports(library),
            library: lower_library_directive(library),
            library_annotations: lower_library_annotations(library),
            import_directives: lower_import_directives(library),
            export_directives: lower_export_directives(library),
            part_directives: lower_part_directives(library),
            part_of: lower_part_of_directive(library),
            span: library.span,
            classes,
            mixins: lower_mixins(library, &mut diagnostics),
            extensions: lower_extensions(library, &mut diagnostics),
            extension_types: lower_extension_types(library, &mut diagnostics),
            functions: lower_functions(library, &mut diagnostics),
            variables: lower_variables(library, &mut diagnostics),
            typedefs: lower_typedefs(library, &mut diagnostics),
            enums,
            query_calls: lower_query_calls(library, &mut diagnostics),
        },
        diagnostics,
    }
}

/// Returns classes that must be lowered because plugins or converters reference them.
fn lowering_required_class_names(library: &ResolvedLibrary) -> HashSet<&str> {
    let mut names = library
        .classes
        .iter()
        .filter(|class| !class.traits.is_empty() || !class.configs.is_empty())
        .map(|class| class.name.as_str())
        .collect::<HashSet<_>>();

    for class in &library.classes {
        for field in &class.fields {
            for config in &field.configs {
                if let Some(converter) = config
                    .named_argument_source("tryFrom")
                    .and_then(try_from_converter_name)
                {
                    names.insert(converter);
                }
            }
        }
    }

    names
}

/// Extracts a converter class name from a `tryFrom` annotation expression.
fn try_from_converter_name(source: &str) -> Option<&str> {
    let value = source.trim();
    let value = value.strip_prefix("const ").unwrap_or(value).trim();
    let before_args = value.split_once('(').map_or(value, |(name, _)| name).trim();
    before_args
        .rsplit('.')
        .next()
        .filter(|name| !name.is_empty())
}

/// Collects import URIs for backwards-compatible plugin input.
fn library_imports(library: &ResolvedLibrary) -> Vec<String> {
    library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Import { uri, .. } => Some(uri.clone()),
            _ => None,
        })
        .collect()
}

/// Lowers the Dart `library` directive, if present.
fn lower_library_directive(library: &ResolvedLibrary) -> Option<LibraryDeclIr> {
    let file_id = library.span.file_id;
    library
        .directives
        .iter()
        .find_map(|directive| match directive {
            ParsedDirective::Library { name, span, .. } => Some(LibraryDeclIr {
                name: name
                    .as_deref()
                    .map(|name| lower_name_ir(file_id, name, *span)),
                span: SpanIr::new(file_id, *span),
            }),
            _ => None,
        })
}

/// Lowers annotations attached to the Dart `library` directive.
fn lower_library_annotations(library: &ResolvedLibrary) -> Vec<AnnotationIr> {
    let file_id = library.span.file_id;
    library
        .directives
        .iter()
        .find_map(|directive| match directive {
            ParsedDirective::Library { annotations, .. } => Some(annotations),
            _ => None,
        })
        .into_iter()
        .flatten()
        .map(|annotation| lower_annotation_ir(file_id, annotation))
        .collect()
}

/// Lowers Dart import directives including combinators and deferred prefixes.
fn lower_import_directives(library: &ResolvedLibrary) -> Vec<ImportIr> {
    let file_id = library.span.file_id;
    library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Import {
                uri,
                prefix,
                show,
                hide,
                is_deferred,
                span,
            } => Some(ImportIr {
                uri: uri.clone(),
                prefix: prefix.clone(),
                show: show.clone(),
                hide: hide.clone(),
                is_deferred: *is_deferred,
                span: SpanIr::new(file_id, *span),
            }),
            _ => None,
        })
        .collect()
}

/// Lowers Dart export directives.
fn lower_export_directives(library: &ResolvedLibrary) -> Vec<ExportIr> {
    let file_id = library.span.file_id;
    library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Export { uri, span } => Some(ExportIr {
                uri: uri.clone(),
                span: SpanIr::new(file_id, *span),
            }),
            _ => None,
        })
        .collect()
}

/// Lowers Dart part directives.
fn lower_part_directives(library: &ResolvedLibrary) -> Vec<PartIr> {
    let file_id = library.span.file_id;
    library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Part { uri, span } => Some(PartIr {
                uri: uri.clone(),
                span: SpanIr::new(file_id, *span),
            }),
            _ => None,
        })
        .collect()
}

/// Lowers the Dart part-of directive, if present.
fn lower_part_of_directive(library: &ResolvedLibrary) -> Option<PartOfIr> {
    let file_id = library.span.file_id;
    library
        .directives
        .iter()
        .find_map(|directive| match directive {
            ParsedDirective::PartOf {
                library_name,
                uri,
                span,
            } => Some(PartOfIr {
                library_name: library_name
                    .as_deref()
                    .map(|name| lower_name_ir(file_id, name, *span)),
                uri: uri.clone(),
                span: SpanIr::new(file_id, *span),
            }),
            _ => None,
        })
}

/// Lowers parsed mixins and their unresolved fields.
fn lower_mixins(library: &ResolvedLibrary, diagnostics: &mut Vec<Diagnostic>) -> Vec<MixinIr> {
    let file_id = library.span.file_id;
    library
        .mixins
        .iter()
        .map(|mixin| MixinIr {
            name: lower_name_ir(file_id, &mixin.name, mixin.span),
            annotations: mixin
                .annotations
                .iter()
                .map(|annotation| lower_annotation_ir(file_id, annotation))
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
fn lower_extensions(
    library: &ResolvedLibrary,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ExtensionIr> {
    let file_id = library.span.file_id;
    library
        .extensions
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
                    .map(|annotation| lower_annotation_ir(file_id, annotation))
                    .collect(),
                span: SpanIr::new(file_id, extension.span),
            }
        })
        .collect()
}

/// Lowers parsed extension types and their representation field.
fn lower_extension_types(
    library: &ResolvedLibrary,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ExtensionTypeIr> {
    let file_id = library.span.file_id;
    library
        .extension_types
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
                    .map(|annotation| lower_annotation_ir(file_id, annotation))
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
fn lower_functions(
    library: &ResolvedLibrary,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<FunctionIr> {
    let file_id = library.span.file_id;
    library
        .functions
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
                    .map(|annotation| lower_annotation_ir(file_id, annotation))
                    .collect(),
                span: SpanIr::new(file_id, function.span),
            }
        })
        .collect()
}

/// Lowers parsed top-level variables and initializers.
fn lower_variables(
    library: &ResolvedLibrary,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<TopLevelVariableIr> {
    let file_id = library.span.file_id;
    library
        .variables
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
                    .map(|annotation| lower_annotation_ir(file_id, annotation))
                    .collect(),
                span: SpanIr::new(file_id, variable.span),
            }
        })
        .collect()
}

/// Lowers parsed typedefs and aliased type sources.
fn lower_typedefs(library: &ResolvedLibrary, diagnostics: &mut Vec<Diagnostic>) -> Vec<TypedefIr> {
    let file_id = library.span.file_id;
    library
        .typedefs
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
                    .map(|annotation| lower_annotation_ir(file_id, annotation))
                    .collect(),
                span: SpanIr::new(file_id, typedef.span),
            }
        })
        .collect()
}

/// Lowers a parsed field before resolver trait/config data exists.
fn lower_unresolved_field(
    file_id: dust_text::FileId,
    field: &ParsedFieldSurface,
    diagnostics: &mut Vec<Diagnostic>,
) -> FieldIr {
    let ty = lower_type(field.parsed_type.as_ref(), field.type_source.as_deref());
    diagnostics.extend(ty.diagnostics);

    FieldIr {
        name: field.name.clone(),
        ty: ty.value,
        span: SpanIr::new(file_id, field.span),
        has_default: field.has_default,
        serde: None,
        configs: Vec::new(),
    }
}

/// Lowers parsed method parameters before resolver trait/config data exists.
fn lower_unresolved_method_params(
    file_id: dust_text::FileId,
    params: &[ParsedMethodParamSurface],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<MethodParamIr> {
    params
        .iter()
        .map(|param| {
            let ty = lower_type(param.parsed_type.as_ref(), param.type_source.as_deref());
            diagnostics.extend(ty.diagnostics);

            MethodParamIr {
                name: param.name.clone(),
                ty: ty.value,
                span: SpanIr::new(file_id, param.span),
                kind: lower_parameter_kind(param.kind),
                has_default: param.has_default,
                default_value_source: param.default_value_source.clone(),
                traits: Vec::new(),
                configs: Vec::new(),
            }
        })
        .collect()
}

/// Maps parser parameter kind to IR parameter kind.
fn lower_parameter_kind(kind: ParameterKind) -> ParamKind {
    match kind {
        ParameterKind::Positional => ParamKind::Positional,
        ParameterKind::Named => ParamKind::Named,
    }
}

/// Lowers a parsed annotation into resolver-compatible annotation IR.
fn lower_annotation_ir(file_id: dust_text::FileId, annotation: &ParsedAnnotation) -> AnnotationIr {
    dust_resolver::annotation_ir_from_parsed(file_id, annotation, None)
}

/// Builds a name IR value from raw source and source span.
fn lower_name_ir(file_id: dust_text::FileId, source: &str, span: dust_text::TextRange) -> NameIr {
    let source = source.trim().to_owned();
    let (prefix, short) = source
        .rsplit_once('.')
        .map(|(prefix, short)| (Some(prefix.to_owned()), short.to_owned()))
        .unwrap_or_else(|| (None, source.clone()));

    NameIr {
        source,
        short,
        prefix,
        span: SpanIr::new(file_id, span),
    }
}

/// Lowers one resolved enum into semantic IR.
fn lower_enum(e: &dust_resolver::ResolvedEnum) -> LoweringOutcome<EnumIr> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let serde = lower_class_serde_config(&e.name, &e.configs, &mut diagnostics);
    let variants: Vec<EnumVariantIr> = e
        .variants
        .iter()
        .map(|v| EnumVariantIr {
            name: v.name.clone(),
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

/// Lowers one resolved class into semantic IR.
fn lower_class(class: &ResolvedClass) -> LoweringOutcome<ClassIr> {
    let mut diagnostics = Vec::new();
    let serde = lower_class_serde_config(&class.name, &class.configs, &mut diagnostics);

    let fields = class
        .fields
        .iter()
        .map(|field| {
            let outcome = lower_type(field.parsed_type.as_ref(), field.type_source.as_deref());
            diagnostics.extend(outcome.diagnostics);
            FieldIr {
                name: field.name.clone(),
                ty: outcome.value,
                span: field.span,
                has_default: field.has_default,
                serde: lower_field_serde_config(&field.name, &field.configs, &mut diagnostics),
                configs: field.configs.clone(),
            }
        })
        .collect::<Vec<_>>();

    let methods = class
        .methods
        .iter()
        .map(|method| {
            let return_type_outcome = lower_type(
                method.surface.parsed_return_type.as_ref(),
                method.surface.return_type_source.as_deref(),
            );
            diagnostics.extend(return_type_outcome.diagnostics);

            let params = method
                .params
                .iter()
                .map(|param| {
                    let type_outcome = lower_type(
                        param.surface.parsed_type.as_ref(),
                        param.surface.type_source.as_deref(),
                    );
                    diagnostics.extend(type_outcome.diagnostics);

                    MethodParamIr {
                        name: param.surface.name.clone(),
                        ty: type_outcome.value,
                        span: param.span,
                        kind: match param.surface.kind {
                            ParameterKind::Positional => ParamKind::Positional,
                            ParameterKind::Named => ParamKind::Named,
                        },
                        has_default: param.surface.has_default,
                        default_value_source: param.surface.default_value_source.clone(),
                        traits: param.traits.clone(),
                        configs: param.configs.clone(),
                    }
                })
                .collect();

            MethodIr {
                name: method.surface.name.clone(),
                is_static: method.surface.is_static,
                is_external: method.surface.is_external,
                return_type: return_type_outcome.value,
                has_body: method.surface.has_body,
                body_source: method.surface.body_source.clone(),
                params,
                span: method.span,
                traits: method.traits.clone(),
                configs: method.configs.clone(),
            }
        })
        .collect();

    let constructors = class
        .constructors
        .iter()
        .map(|constructor| {
            let params = constructor
                .params
                .iter()
                .map(|param| {
                    let outcome = param
                        .type_source
                        .as_deref()
                        .map(|source| lower_type(param.parsed_type.as_ref(), Some(source)))
                        .unwrap_or_else(|| infer_param_type(param.name.as_str(), &fields));
                    diagnostics.extend(outcome.diagnostics);
                    ConstructorParamIr {
                        name: param.name.clone(),
                        ty: outcome.value,
                        span: SpanIr::new(class.span.file_id, param.span),
                        kind: match param.kind {
                            ParameterKind::Positional => ParamKind::Positional,
                            ParameterKind::Named => ParamKind::Named,
                        },
                        has_default: param.has_default,
                        default_value_source: param.default_value_source.clone(),
                    }
                })
                .collect();

            ConstructorIr {
                name: constructor.name.clone(),
                is_factory: constructor.is_factory,
                redirected_target_source: constructor.redirected_target_source.clone(),
                redirected_target_name: constructor.redirected_target_name.clone(),
                span: SpanIr::new(class.span.file_id, constructor.span),
                params,
            }
        })
        .collect();

    LoweringOutcome {
        value: ClassIr {
            kind: class.kind,
            name: class.name.clone(),
            is_abstract: class.is_abstract,
            is_interface: class.is_interface,
            superclass_name: class.superclass_name.clone(),
            span: class.span,
            fields,
            constructors,
            methods,
            traits: class.traits.clone(),
            configs: class.configs.clone(),
            serde,
        },
        diagnostics,
    }
}

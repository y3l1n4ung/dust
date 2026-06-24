use std::collections::BTreeMap;

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{AnnotationValueIr, ConfigApplicationIr, SpanIr, SymbolId, TraitApplicationIr};
use dust_parser_dart::{
    ParsedAnnotation, ParsedConstructorSurface, ParsedDirective, ParsedFieldSurface,
    ParsedMethodParamSurface, ParsedMethodSurface,
};
use dust_text::{FileId, TextRange};

use crate::{
    ResolvedConstructor, ResolvedField, ResolvedMethod, ResolvedMethodParam, SymbolCatalog,
    SymbolKind, annotations::annotation_argument_values,
};

/// Positional and named annotation argument values.
type AnnotationArguments = (Vec<AnnotationValueIr>, BTreeMap<String, AnnotationValueIr>);

/// One parsed annotation matched to a registered symbol.
struct ResolvedAnnotationSymbol {
    /// Annotation source span.
    span: TextRange,
    /// Matched symbol kind.
    kind: SymbolKind,
    /// Fully qualified symbol id.
    symbol: SymbolId,
    /// Original argument source when available.
    arguments_source: Option<String>,
    /// Lowered annotation arguments.
    arguments: AnnotationArguments,
}

/// Resolves annotations and parameters for one method.
pub(crate) fn resolve_method(
    file_id: FileId,
    method: &ParsedMethodSurface,
    catalog: &SymbolCatalog,
    _diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedMethod {
    let mut traits = Vec::new();
    let mut configs = Vec::new();

    for annotation in &method.annotations {
        let Some(resolved) = catalog
            .resolve_config(&annotation.name)
            .or_else(|| catalog.resolve_trait(&annotation.name))
        else {
            continue;
        };

        push_resolved_symbol(
            file_id,
            ResolvedAnnotationSymbol {
                span: annotation.span,
                kind: resolved.kind,
                symbol: resolved.symbol.clone(),
                arguments_source: annotation.arguments_source.clone(),
                arguments: annotation_argument_values(file_id, annotation),
            },
            &mut traits,
            &mut configs,
        );
    }

    let params = method
        .params
        .iter()
        .map(|param| resolve_method_param(file_id, param, catalog))
        .collect();

    ResolvedMethod {
        surface: method.clone(),
        span: SpanIr::new(file_id, method.span),
        traits,
        configs,
        params,
    }
}

/// Resolves annotations for one method parameter.
fn resolve_method_param(
    file_id: FileId,
    param: &ParsedMethodParamSurface,
    catalog: &SymbolCatalog,
) -> ResolvedMethodParam {
    let mut traits = Vec::new();
    let mut configs = Vec::new();

    for annotation in &param.annotations {
        let Some(resolved) = catalog.resolve(&annotation.name) else {
            continue;
        };

        push_resolved_symbol(
            file_id,
            ResolvedAnnotationSymbol {
                span: annotation.span,
                kind: resolved.kind,
                symbol: resolved.symbol.clone(),
                arguments_source: annotation.arguments_source.clone(),
                arguments: annotation_argument_values(file_id, annotation),
            },
            &mut traits,
            &mut configs,
        );
    }

    ResolvedMethodParam {
        surface: param.clone(),
        span: SpanIr::new(file_id, param.span),
        traits,
        configs,
    }
}

/// Resolves declaration-level Dust trait and config annotations.
pub(crate) fn resolve_declaration_annotations(
    file_id: FileId,
    annotations: &[ParsedAnnotation],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
    traits: &mut Vec<TraitApplicationIr>,
    configs: &mut Vec<ConfigApplicationIr>,
) {
    for annotation in annotations {
        if annotation.is_named("Derive") {
            for name in annotation.positional_constructor_names() {
                match catalog.resolve_trait(&name) {
                    Some(resolved) => push_resolved_symbol(
                        file_id,
                        ResolvedAnnotationSymbol {
                            span: annotation.span,
                            kind: resolved.kind,
                            symbol: resolved.symbol.clone(),
                            arguments_source: None,
                            arguments: (Vec::new(), BTreeMap::new()),
                        },
                        traits,
                        configs,
                    ),
                    None => diagnostics.push(
                        Diagnostic::warning(format!("unknown derive trait or config `{name}`"))
                            .with_label(SourceLabel::new(
                                file_id,
                                annotation.span,
                                "annotation member is not owned by any registered symbol",
                            )),
                    ),
                }
            }
        } else if let Some(resolved) = catalog
            .resolve_config(&annotation.name)
            .or_else(|| catalog.resolve_trait(&annotation.name))
        {
            push_resolved_symbol(
                file_id,
                ResolvedAnnotationSymbol {
                    span: annotation.span,
                    kind: resolved.kind,
                    symbol: resolved.symbol.clone(),
                    arguments_source: annotation.arguments_source.clone(),
                    arguments: annotation_argument_values(file_id, annotation),
                },
                traits,
                configs,
            );
        }
    }
}

/// Resolves constructor-level Dust config annotations.
pub(crate) fn resolve_constructor(
    file_id: FileId,
    constructor: &ParsedConstructorSurface,
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedConstructor {
    let mut configs = Vec::new();

    for annotation in &constructor.annotations {
        let Some(resolved) = catalog.resolve_config(&annotation.name) else {
            if catalog.resolve_trait(&annotation.name).is_some() {
                diagnostics.push(
                    Diagnostic::warning(format!(
                        "trait annotation `{}` is not supported on constructors",
                        annotation.name
                    ))
                    .with_label(SourceLabel::new(
                        file_id,
                        annotation.span,
                        "constructor annotations may only use Dust config symbols",
                    )),
                );
            }
            continue;
        };

        let (positional_args, named_args) = annotation_argument_values(file_id, annotation);
        configs.push(ConfigApplicationIr::with_arguments(
            resolved.symbol.clone(),
            annotation.arguments_source.clone(),
            positional_args,
            named_args,
            SpanIr::new(file_id, annotation.span),
        ));
    }

    ResolvedConstructor {
        surface: constructor.clone(),
        configs,
    }
}

/// Resolves field-level Dust config annotations.
pub(crate) fn resolve_field(
    file_id: FileId,
    field: &ParsedFieldSurface,
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedField {
    let mut configs = Vec::new();

    for annotation in &field.annotations {
        let Some(resolved) = catalog.resolve_config(&annotation.name) else {
            if catalog.resolve_trait(&annotation.name).is_some() {
                diagnostics.push(
                    Diagnostic::warning(format!(
                        "trait annotation `{}` is not supported on fields",
                        annotation.name
                    ))
                    .with_label(SourceLabel::new(
                        file_id,
                        annotation.span,
                        "field annotations may only use Dust config symbols",
                    )),
                );
            }
            continue;
        };

        let (positional_args, named_args) = annotation_argument_values(file_id, annotation);
        configs.push(ConfigApplicationIr::with_arguments(
            resolved.symbol.clone(),
            annotation.arguments_source.clone(),
            positional_args,
            named_args,
            SpanIr::new(file_id, annotation.span),
        ));
    }

    ResolvedField {
        name: field.name.clone(),
        type_source: field.type_source.clone(),
        parsed_type: field.parsed_type.clone(),
        has_default: field.has_default,
        span: SpanIr::new(file_id, field.span),
        configs,
    }
}

/// Returns the first generated part URI from parsed directives.
pub(crate) fn first_part_uri(directives: &[ParsedDirective]) -> Option<String> {
    directives.iter().find_map(|directive| match directive {
        ParsedDirective::Part { uri, .. } => Some(uri.clone()),
        _ => None,
    })
}

/// Pushes one resolved symbol into the matching trait or config list.
fn push_resolved_symbol(
    file_id: FileId,
    application: ResolvedAnnotationSymbol,
    traits: &mut Vec<TraitApplicationIr>,
    configs: &mut Vec<ConfigApplicationIr>,
) {
    match application.kind {
        SymbolKind::Trait => traits.push(TraitApplicationIr {
            symbol: application.symbol,
            span: SpanIr::new(file_id, application.span),
        }),
        SymbolKind::Config => {
            let (positional_args, named_args) = application.arguments;
            configs.push(ConfigApplicationIr::with_arguments(
                application.symbol,
                application.arguments_source,
                positional_args,
                named_args,
                SpanIr::new(file_id, application.span),
            ));
        }
    }
}

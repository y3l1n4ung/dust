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
    SymbolKind, annotations::annotation_argument_values, serde::normalize_field_serde,
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
        let Some(resolved) = resolve_annotation(catalog, annotation) else {
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
        let Some(resolved) = resolve_annotation(catalog, annotation) else {
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
            let (positional, _) = annotation_argument_values(file_id, annotation);
            for name in derive_member_names(&positional) {
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
        } else if let Some(resolved) = resolve_annotation(catalog, annotation) {
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

/// Returns constructor names from structured `Derive` positional values.
fn derive_member_names(values: &[AnnotationValueIr]) -> Vec<String> {
    values
        .iter()
        .flat_map(|value| match value {
            AnnotationValueIr::List(items) => items.as_slice(),
            value => std::slice::from_ref(value),
        })
        .filter_map(|value| match value {
            AnnotationValueIr::Constructor { name, .. } => Some(name.short.clone()),
            _ => None,
        })
        .collect()
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
        let Some(resolved) = resolve_annotation(catalog, annotation)
            .filter(|resolved| resolved.kind == SymbolKind::Config)
        else {
            if resolve_annotation_trait(catalog, annotation).is_some() {
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
        let Some(resolved) = resolve_annotation(catalog, annotation)
            .filter(|resolved| resolved.kind == SymbolKind::Config)
        else {
            if resolve_annotation_trait(catalog, annotation).is_some() {
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

    let serde = normalize_field_serde(&field.name, &configs, diagnostics);
    ResolvedField {
        name: field.name.clone(),
        type_source: field.type_source.clone(),
        parsed_type: field.parsed_type.clone(),
        has_default: field.has_default,
        span: SpanIr::new(file_id, field.span),
        configs,
        serde,
    }
}

/// Resolves an annotation by canonical symbol identity, then short-name compatibility.
fn resolve_annotation<'a>(
    catalog: &'a SymbolCatalog,
    annotation: &ParsedAnnotation,
) -> Option<&'a crate::ResolvedSymbol> {
    catalog
        .resolve_qualified_config(&annotation.qualified_name)
        .or_else(|| catalog.resolve_config(&annotation.name))
        .or_else(|| catalog.resolve_qualified_trait(&annotation.qualified_name))
        .or_else(|| catalog.resolve_trait(&annotation.name))
}

/// Resolves a trait annotation by canonical name, then short-name compatibility.
fn resolve_annotation_trait<'a>(
    catalog: &'a SymbolCatalog,
    annotation: &ParsedAnnotation,
) -> Option<&'a crate::ResolvedSymbol> {
    catalog
        .resolve_qualified_trait(&annotation.qualified_name)
        .or_else(|| catalog.resolve_trait(&annotation.name))
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::derive_member_names;
    use dust_ir::{AnnotationValueIr, NameIr, SpanIr};
    use dust_text::{FileId, TextRange};

    #[test]
    fn derive_members_come_from_structured_constructor_values() {
        let span = SpanIr::new(FileId::new(1), TextRange::new(0_u32, 10_u32));
        let values = vec![AnnotationValueIr::List(vec![
            AnnotationValueIr::Constructor {
                name: NameIr {
                    source: "d.ToString".to_owned(),
                    short: "ToString".to_owned(),
                    prefix: Some("d".to_owned()),
                    span,
                },
                positional_args: Vec::new(),
                named_args: BTreeMap::new(),
            },
            AnnotationValueIr::Expression(dust_ir::ExprSourceIr {
                source: "Unknown()".to_owned(),
                span,
            }),
        ])];

        assert_eq!(derive_member_names(&values), ["ToString"]);
    }
}

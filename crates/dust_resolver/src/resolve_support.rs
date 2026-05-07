use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ConfigApplicationIr, SpanIr, TraitApplicationIr};
use dust_parser_dart::{
    ParsedAnnotation, ParsedDirective, ParsedFieldSurface, ParsedMethodParamSurface,
    ParsedMethodSurface,
};
use dust_text::{FileId, TextRange};

use crate::{ResolvedField, ResolvedMethod, ResolvedMethodParam, SymbolCatalog, SymbolKind};

pub(crate) fn resolve_method(
    file_id: FileId,
    method: &ParsedMethodSurface,
    catalog: &SymbolCatalog,
    _diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedMethod {
    let mut traits = Vec::new();
    let mut configs = Vec::new();

    for annotation in &method.annotations {
        let Some(resolved) = catalog.resolve(&annotation.name) else {
            continue;
        };

        push_resolved_symbol(
            file_id,
            annotation.span,
            resolved.kind,
            resolved.symbol.clone(),
            annotation.arguments_source.clone(),
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
            annotation.span,
            resolved.kind,
            resolved.symbol.clone(),
            annotation.arguments_source.clone(),
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

pub(crate) fn resolve_declaration_annotations(
    file_id: FileId,
    annotations: &[ParsedAnnotation],
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
    traits: &mut Vec<TraitApplicationIr>,
    configs: &mut Vec<ConfigApplicationIr>,
) {
    for annotation in annotations {
        if annotation.name == "Derive" {
            for name in derive_member_names(annotation.arguments_source.as_deref().unwrap_or("")) {
                match catalog.resolve(&name) {
                    Some(resolved) => push_resolved_symbol(
                        file_id,
                        annotation.span,
                        resolved.kind,
                        resolved.symbol.clone(),
                        None,
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
        } else if let Some(resolved) = catalog.resolve(&annotation.name) {
            push_resolved_symbol(
                file_id,
                annotation.span,
                resolved.kind,
                resolved.symbol.clone(),
                annotation.arguments_source.clone(),
                traits,
                configs,
            );
        }
    }
}

pub(crate) fn resolve_field(
    file_id: FileId,
    field: &ParsedFieldSurface,
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedField {
    let mut configs = Vec::new();

    for annotation in &field.annotations {
        let Some(resolved) = catalog.resolve(&annotation.name) else {
            continue;
        };

        match resolved.kind {
            SymbolKind::Config => configs.push(ConfigApplicationIr {
                symbol: resolved.symbol.clone(),
                arguments_source: annotation.arguments_source.clone(),
                span: SpanIr::new(file_id, annotation.span),
            }),
            SymbolKind::Trait => diagnostics.push(
                Diagnostic::warning(format!(
                    "trait annotation `{}` is not supported on fields",
                    annotation.name
                ))
                .with_label(SourceLabel::new(
                    file_id,
                    annotation.span,
                    "field annotations may only use Dust config symbols",
                )),
            ),
        }
    }

    ResolvedField {
        name: field.name.clone(),
        type_source: field.type_source.clone(),
        has_default: field.has_default,
        span: SpanIr::new(file_id, field.span),
        configs,
    }
}

pub(crate) fn first_part_uri(directives: &[ParsedDirective]) -> Option<String> {
    directives.iter().find_map(|directive| match directive {
        ParsedDirective::Part { uri, .. } => Some(uri.clone()),
        _ => None,
    })
}

fn push_resolved_symbol(
    file_id: FileId,
    span: TextRange,
    kind: SymbolKind,
    symbol: dust_ir::SymbolId,
    arguments_source: Option<String>,
    traits: &mut Vec<TraitApplicationIr>,
    configs: &mut Vec<ConfigApplicationIr>,
) {
    match kind {
        SymbolKind::Trait => traits.push(TraitApplicationIr {
            symbol,
            span: SpanIr::new(file_id, span),
        }),
        SymbolKind::Config => configs.push(ConfigApplicationIr {
            symbol,
            arguments_source,
            span: SpanIr::new(file_id, span),
        }),
    }
}

fn derive_member_names(arguments_source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut chars = arguments_source.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '_' || ch.is_ascii_alphabetic() {
            let mut ident = String::from(ch);
            while let Some(next) = chars.peek() {
                if *next == '_' || next.is_ascii_alphanumeric() {
                    ident.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }

            if chars.peek().copied() == Some('(') {
                names.push(ident);
            }
        }
    }

    names
}

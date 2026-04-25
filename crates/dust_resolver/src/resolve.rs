use std::path::Path;

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ClassKindIr, ConfigApplicationIr, SpanIr, TraitApplicationIr};
use dust_parser_dart::{
    ParsedClassKind, ParsedClassSurface, ParsedDirective, ParsedFieldSurface, ParsedLibrarySurface,
};
use dust_text::{FileId, TextRange};

use crate::{
    ResolveResult, ResolvedClass, ResolvedField, ResolvedLibrary, SymbolCatalog, SymbolKind,
};

/// Resolves one parsed library against a symbol catalog.
pub fn resolve_library(
    file_id: FileId,
    source_path: &str,
    library: &ParsedLibrarySurface,
    catalog: &SymbolCatalog,
) -> ResolveResult {
    let mut diagnostics = Vec::new();
    let output_path = expected_output_path(source_path);
    let part_uri = first_part_uri(&library.directives);

    let mut classes = Vec::new();
    let mut saw_dust_symbol = false;

    for class in &library.classes {
        let resolved = resolve_class(file_id, class, catalog, &mut diagnostics);
        if !resolved.traits.is_empty() || !resolved.configs.is_empty() {
            saw_dust_symbol = true;
        }
        classes.push(resolved);
    }

    if saw_dust_symbol {
        match part_uri.as_deref() {
            Some(uri) => {
                if let Err(diagnostic) = validate_generated_part_uri(source_path, uri) {
                    diagnostics.push(diagnostic);
                }
            }
            None => diagnostics.push(
                Diagnostic::error("missing generated `part` directive for Dust-enabled library")
                    .with_label(SourceLabel::new(
                        file_id,
                        library.span,
                        "expected a matching `part 'file.g.dart';` directive",
                    )),
            ),
        }
    }

    ResolveResult {
        library: ResolvedLibrary {
            source_path: source_path.to_owned(),
            output_path,
            span: SpanIr::new(file_id, library.span),
            directives: library.directives.clone(),
            part_uri,
            classes,
        },
        diagnostics,
    }
}

/// Validates that a generated part URI matches the source file name.
pub fn validate_generated_part_uri(source_path: &str, part_uri: &str) -> Result<(), Diagnostic> {
    let Some(stem) = Path::new(source_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
    else {
        return Err(Diagnostic::error(
            "source path must contain a valid file stem for generated output",
        ));
    };

    let expected = format!("{stem}.g.dart");
    if !part_uri.ends_with(".g.dart") {
        return Err(Diagnostic::error(
            "generated part path must end with `.g.dart`",
        ));
    }

    let Some(file_name) = Path::new(part_uri)
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return Err(Diagnostic::error(
            "generated part path must contain a valid file name",
        ));
    };

    if file_name != expected {
        return Err(Diagnostic::error(format!(
            "generated part path `{file_name}` does not match expected `{expected}`"
        )));
    }

    Ok(())
}

fn resolve_class(
    file_id: FileId,
    class: &ParsedClassSurface,
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedClass {
    let mut traits = Vec::new();
    let mut configs = Vec::new();

    for annotation in &class.annotations {
        if annotation.name == "Derive" {
            for name in derive_member_names(annotation.arguments_source.as_deref().unwrap_or("")) {
                match catalog.resolve(&name) {
                    Some(resolved) => push_resolved_symbol(
                        file_id,
                        annotation.span,
                        resolved.kind,
                        resolved.symbol.clone(),
                        None,
                        &mut traits,
                        &mut configs,
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
                &mut traits,
                &mut configs,
            );
        }
    }

    let fields = class
        .fields
        .iter()
        .map(|field| resolve_field(file_id, field, catalog, diagnostics))
        .collect();

    ResolvedClass {
        kind: match class.kind {
            ParsedClassKind::Class => ClassKindIr::Class,
            ParsedClassKind::MixinClass => ClassKindIr::MixinClass,
        },
        name: class.name.clone(),
        is_abstract: class.is_abstract,
        superclass_name: class.superclass_name.clone(),
        span: SpanIr::new(file_id, class.span),
        fields,
        constructors: class.constructors.clone(),
        traits,
        configs,
    }
}

fn resolve_field(
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

fn first_part_uri(directives: &[ParsedDirective]) -> Option<String> {
    directives.iter().find_map(|directive| match directive {
        ParsedDirective::Part { uri, .. } => Some(uri.clone()),
        _ => None,
    })
}

fn expected_output_path(source_path: &str) -> String {
    let path = Path::new(source_path);
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("file");
    let file_name = format!("{stem}.g.dart");
    path.with_file_name(file_name)
        .to_string_lossy()
        .into_owned()
}

use std::path::Path;

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ClassKindIr, ConfigApplicationIr, SpanIr, TraitApplicationIr};
use dust_parser_dart::{ParsedClassKind, ParsedClassSurface, ParsedLibrarySurface};
use dust_text::FileId;

use crate::{
    ResolveResult, ResolvedClass, ResolvedEnum, ResolvedEnumVariant, ResolvedLibrary,
    SymbolCatalog,
    resolve_support::{
        first_part_uri, resolve_declaration_annotations, resolve_field, resolve_method,
    },
};

/// Resolves one parsed library against a symbol catalog.
pub fn resolve_library(
    file_id: FileId,
    source_path: &str,
    output_path: &str,
    library: &ParsedLibrarySurface,
    catalog: &SymbolCatalog,
) -> ResolveResult {
    let mut diagnostics = Vec::new();
    let part_uri = first_part_uri(&library.directives);
    let mut enums: Vec<ResolvedEnum> = Vec::new();
    let mut classes = Vec::new();
    let mut saw_dust_symbol = false;

    for class in &library.classes {
        let resolved = resolve_class(file_id, class, catalog, &mut diagnostics);
        if !resolved.traits.is_empty() || !resolved.configs.is_empty() {
            saw_dust_symbol = true;
        }
        classes.push(resolved);
    }
    for enum_surface in &library.enums {
        let resolved: ResolvedEnum = resolve_enum(file_id, enum_surface, catalog, &mut diagnostics);
        if !resolved.traits.is_empty() || !resolved.configs.is_empty() {
            saw_dust_symbol = true;
        }
        enums.push(resolved);
    }

    if saw_dust_symbol {
        match part_uri.as_deref() {
            Some(uri) => {
                if let Err(diagnostic) = validate_generated_part_uri(output_path, uri) {
                    diagnostics.push(diagnostic);
                }
            }
            None => diagnostics.push(
                Diagnostic::error("missing generated `part` directive for Dust-enabled library")
                    .with_label(SourceLabel::new(
                        file_id,
                        library.span,
                        format!(
                            "expected a matching `part '{}';` directive",
                            expected_part_uri(output_path)
                        ),
                    )),
            ),
        }
    }

    ResolveResult {
        library: ResolvedLibrary {
            source_path: source_path.to_owned(),
            output_path: output_path.to_owned(),
            span: SpanIr::new(file_id, library.span),
            directives: library.directives.clone(),
            part_uri,
            classes,
            enums,
        },
        diagnostics,
    }
}

fn resolve_enum(
    file_id: FileId,
    enum_surface: &dust_parser_dart::ParsedEnumSurface,
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedEnum {
    let mut traits: Vec<TraitApplicationIr> = Vec::new();
    let mut configs: Vec<ConfigApplicationIr> = Vec::new();
    resolve_declaration_annotations(
        file_id,
        &enum_surface.annotations,
        catalog,
        diagnostics,
        &mut traits,
        &mut configs,
    );

    let variants = enum_surface
        .variants
        .iter()
        .map(|variant| ResolvedEnumVariant {
            name: variant.name.clone(),
            span: SpanIr::new(file_id, variant.span),
        })
        .collect();

    ResolvedEnum {
        name: enum_surface.name.clone(),
        span: SpanIr::new(file_id, enum_surface.span),
        variants,
        traits,
        configs,
    }
}

/// Validates that a generated part URI matches the source file name.
pub fn validate_generated_part_uri(output_path: &str, part_uri: &str) -> Result<(), Diagnostic> {
    let expected = expected_part_uri(output_path);
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

fn expected_part_uri(output_path: &str) -> String {
    Path::new(output_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file.g.dart")
        .to_owned()
}

fn resolve_class(
    file_id: FileId,
    class: &ParsedClassSurface,
    catalog: &SymbolCatalog,
    diagnostics: &mut Vec<Diagnostic>,
) -> ResolvedClass {
    let mut traits = Vec::new();
    let mut configs = Vec::new();

    resolve_declaration_annotations(
        file_id,
        &class.annotations,
        catalog,
        diagnostics,
        &mut traits,
        &mut configs,
    );

    let fields = class
        .fields
        .iter()
        .map(|field| resolve_field(file_id, field, catalog, diagnostics))
        .collect();

    let methods = class
        .methods
        .iter()
        .map(|method| resolve_method(file_id, method, catalog, diagnostics))
        .collect();

    ResolvedClass {
        kind: match class.kind {
            ParsedClassKind::Class => ClassKindIr::Class,
            ParsedClassKind::MixinClass => ClassKindIr::MixinClass,
        },
        name: class.name.clone(),
        is_abstract: class.is_abstract,
        is_interface: class.is_interface,
        superclass_name: class.superclass_name.clone(),
        span: SpanIr::new(file_id, class.span),
        fields,
        constructors: class.constructors.clone(),
        methods,
        traits,
        configs,
    }
}

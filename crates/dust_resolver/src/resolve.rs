use std::path::Path;

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ClassKindIr, ConfigApplicationIr, SpanIr, TraitApplicationIr};
use dust_parser_dart::{ParsedClassKind, ParsedClassSurface, ParsedDartFileSurface};
use dust_text::FileId;

use crate::{
    ResolveResult, ResolvedClass, ResolvedEnum, ResolvedEnumVariant, ResolvedLibrary,
    SymbolCatalog,
    resolve_support::{
        first_part_uri, resolve_constructor, resolve_declaration_annotations, resolve_field,
        resolve_method,
    },
};

/// Resolves one parsed library against a symbol catalog.
pub fn resolve_library(
    file_id: FileId,
    source_path: &str,
    output_path: &str,
    library: &ParsedDartFileSurface,
    catalog: &SymbolCatalog,
) -> ResolveResult {
    resolve_library_with_partless_configs(file_id, source_path, output_path, library, catalog, &[])
}

/// Resolves one parsed library while allowing selected config symbols to emit standalone outputs.
pub fn resolve_library_with_partless_configs(
    file_id: FileId,
    source_path: &str,
    output_path: &str,
    library: &ParsedDartFileSurface,
    catalog: &SymbolCatalog,
    partless_config_symbols: &[&str],
) -> ResolveResult {
    let mut diagnostics = Vec::new();
    let part_uri = first_part_uri(&library.directives);
    let mut enums: Vec<ResolvedEnum> = Vec::new();
    let mut classes = Vec::new();
    let mut saw_dust_symbol = false;

    for class in &library.classes {
        let resolved = resolve_class(file_id, class, catalog, &mut diagnostics);
        if class_has_dust_symbol(&resolved) {
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

    let needs_part = saw_dust_symbol
        && classes
            .iter()
            .any(|class| class_needs_part(class, partless_config_symbols))
        || enums
            .iter()
            .any(|enum_ir| enum_needs_part(enum_ir, partless_config_symbols));

    if needs_part {
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
            mixins: library.mixins.clone(),
            extensions: library.extensions.clone(),
            extension_types: library.extension_types.clone(),
            functions: library.functions.clone(),
            variables: library.variables.clone(),
            typedefs: library.typedefs.clone(),
            query_calls: library.query_calls.clone(),
        },
        diagnostics,
    }
}

/// Returns whether a resolved class requires a generated part file.
fn class_needs_part(class: &ResolvedClass, partless_config_symbols: &[&str]) -> bool {
    !class.traits.is_empty()
        || class
            .configs
            .iter()
            .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
        || class.constructors.iter().any(|constructor| {
            constructor
                .configs
                .iter()
                .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
        })
        || class.fields.iter().any(|field| {
            field
                .configs
                .iter()
                .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
        })
        || class.methods.iter().any(|method| {
            !method.traits.is_empty()
                || method
                    .configs
                    .iter()
                    .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
                || method.params.iter().any(|param| {
                    !param.traits.is_empty()
                        || param.configs.iter().any(|config| {
                            !partless_config_symbols.contains(&config.symbol.0.as_str())
                        })
                })
        })
}

/// Returns whether a resolved class contains any Dust-owned symbol.
fn class_has_dust_symbol(class: &ResolvedClass) -> bool {
    !class.traits.is_empty()
        || !class.configs.is_empty()
        || class
            .constructors
            .iter()
            .any(|constructor| !constructor.configs.is_empty())
        || class.fields.iter().any(|field| !field.configs.is_empty())
        || class.methods.iter().any(|method| {
            !method.traits.is_empty()
                || !method.configs.is_empty()
                || method
                    .params
                    .iter()
                    .any(|param| !param.traits.is_empty() || !param.configs.is_empty())
        })
}

/// Returns whether a resolved enum requires a generated part file.
fn enum_needs_part(enum_ir: &ResolvedEnum, partless_config_symbols: &[&str]) -> bool {
    !enum_ir.traits.is_empty()
        || enum_ir
            .configs
            .iter()
            .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
}

/// Resolves one parsed enum into semantic data.
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

/// Returns the expected part URI from a generated output path.
fn expected_part_uri(output_path: &str) -> String {
    Path::new(output_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file.g.dart")
        .to_owned()
}

/// Resolves one parsed class into semantic data.
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

    let constructors = class
        .constructors
        .iter()
        .map(|constructor| resolve_constructor(file_id, constructor, catalog, diagnostics))
        .collect();

    ResolvedClass {
        kind: match class.kind {
            ParsedClassKind::Class => ClassKindIr::Class,
            ParsedClassKind::SealedClass => ClassKindIr::SealedClass,
            ParsedClassKind::MixinClass => ClassKindIr::MixinClass,
        },
        name: class.name.clone(),
        is_abstract: class.is_abstract,
        is_interface: class.is_interface,
        superclass_name: class.superclass_name.clone(),
        span: SpanIr::new(file_id, class.span),
        fields,
        constructors,
        methods,
        traits,
        configs,
    }
}

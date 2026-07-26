use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{
    ClassKindIr, ConfigApplicationIr, DbConfigIr, NormalizedConfigIr, SpanIr, TraitApplicationIr,
};
use dust_parser_dart::{
    ParsedAnnotation, ParsedClassKind, ParsedClassSurface, ParsedDartFileSurface, ParsedDirective,
};
use dust_text::FileId;

use crate::{
    ResolveResult, ResolvedClass, ResolvedEnum, ResolvedEnumVariant, ResolvedLibrary,
    SymbolCatalog,
    db::normalize_db,
    http::normalize_http,
    query_calls::normalize_query_calls,
    resolve_support::{
        first_part_uri, resolve_constructor, resolve_declaration_annotations, resolve_field,
        resolve_method,
    },
    route::normalize_route,
    serde::{normalize_class_serde, normalize_enum_variant_serde, normalize_sealed_serde_variants},
    state::normalize_state,
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
    validate_annotation_prefixes(file_id, library, &mut diagnostics);
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
    normalize_sealed_serde_variants(&mut classes, &mut diagnostics);
    mark_required_lowering_diagnostics(&mut classes);

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
            query_calls: normalize_query_calls(file_id, &library.query_calls, &mut diagnostics),
        },
        diagnostics,
    }
}

/// Reports annotation prefixes that are not declared by an import directive.
fn validate_annotation_prefixes(
    file_id: FileId,
    library: &ParsedDartFileSurface,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let prefixes = library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Import {
                prefix: Some(prefix),
                ..
            } => Some(prefix.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    let mut check = |annotations: &[ParsedAnnotation]| {
        for annotation in annotations {
            let Some(prefix) = annotation.prefix.as_deref() else {
                continue;
            };
            if prefixes.contains(prefix) {
                continue;
            }
            diagnostics.push(
                Diagnostic::error(format!(
                    "annotation prefix `{prefix}` is not declared by an import"
                ))
                .with_label(SourceLabel::new(
                    file_id,
                    annotation.span,
                    format!("add `as {prefix}` to the matching import or remove the prefix"),
                )),
            );
        }
    };

    for directive in &library.directives {
        if let ParsedDirective::Library { annotations, .. } = directive {
            check(annotations);
        }
    }
    for class in &library.classes {
        check(&class.annotations);
        for field in &class.fields {
            check(&field.annotations);
        }
        for constructor in &class.constructors {
            check(&constructor.annotations);
            for param in &constructor.params {
                check(&param.annotations);
            }
        }
        for method in &class.methods {
            check(&method.annotations);
            for param in &method.params {
                check(&param.annotations);
            }
        }
    }
    for enum_surface in &library.enums {
        check(&enum_surface.annotations);
        for variant in &enum_surface.variants {
            check(&variant.annotations);
        }
    }
    for mixin in &library.mixins {
        check(&mixin.annotations);
        for field in &mixin.fields {
            check(&field.annotations);
        }
    }
    for extension in &library.extensions {
        check(&extension.annotations);
    }
    for extension_type in &library.extension_types {
        check(&extension_type.annotations);
    }
    for function in &library.functions {
        check(&function.annotations);
        for param in &function.params {
            check(&param.annotations);
        }
    }
    for variable in &library.variables {
        check(&variable.annotations);
    }
    for typedef in &library.typedefs {
        check(&typedef.annotations);
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
        .map(|variant| {
            let mut variant_traits = Vec::new();
            let mut variant_configs = Vec::new();
            resolve_declaration_annotations(
                file_id,
                &variant.annotations,
                catalog,
                diagnostics,
                &mut variant_traits,
                &mut variant_configs,
            );
            ResolvedEnumVariant {
                name: variant.name.clone(),
                span: SpanIr::new(file_id, variant.span),
                serde: normalize_enum_variant_serde(&variant.name, &variant_configs, diagnostics),
                configs: variant_configs,
            }
        })
        .collect();

    ResolvedEnum {
        name: enum_surface.name.clone(),
        span: SpanIr::new(file_id, enum_surface.span),
        variants,
        traits,
        serde: normalize_class_serde(&enum_surface.name, &configs, diagnostics),
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

    let mut fields: Vec<crate::ResolvedField> = class
        .fields
        .iter()
        .map(|field| resolve_field(file_id, field, catalog, diagnostics))
        .collect();

    let mut methods: Vec<crate::ResolvedMethod> = class
        .methods
        .iter()
        .map(|method| resolve_method(file_id, method, catalog, diagnostics))
        .collect();

    let mut constructors: Vec<crate::ResolvedConstructor> = class
        .constructors
        .iter()
        .map(|constructor| resolve_constructor(file_id, constructor, catalog, diagnostics))
        .collect();

    let serde = normalize_class_serde(&class.name, &configs, diagnostics);
    normalize_route(&mut configs);
    normalize_state(&mut configs);
    normalize_http(&mut configs, &mut methods);
    normalize_db(&mut configs, &mut fields, &mut constructors, &mut methods);

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
        serde,
        requires_lowering_diagnostics: false,
    }
}

/// Marks classes whose lowering diagnostics are relevant to generated output.
fn mark_required_lowering_diagnostics(classes: &mut [ResolvedClass]) {
    let mut names = classes
        .iter()
        .filter(|class| !class.traits.is_empty() || !class.configs.is_empty())
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();

    for class in classes.iter() {
        for field in &class.fields {
            for config in &field.configs {
                if let Some(NormalizedConfigIr::Db(DbConfigIr::Sqlx(sqlx))) =
                    config.normalized.as_ref()
                    && let Some(converter) = &sqlx.try_from_class_name
                {
                    names.insert(converter.clone());
                }
            }
        }
        if let Some(serde) = &class.serde {
            for variant in &serde.variants {
                names.insert(variant.target_class_name.clone());
            }
        }
    }

    for class in classes {
        class.requires_lowering_diagnostics = names.contains(&class.name);
    }
}

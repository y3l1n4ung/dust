//! Resolving one class body into the resolver's own representation.

use super::*;

/// Resolves one parsed class into semantic data.
pub(super) fn resolve_class(
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
pub(super) fn mark_required_lowering_diagnostics(classes: &mut [ResolvedClass]) {
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

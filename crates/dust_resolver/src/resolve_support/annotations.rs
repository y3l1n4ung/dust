//! Resolving an annotation and its trait application to catalog symbols.

use super::*;

/// Resolves an annotation by canonical symbol identity, then short-name compatibility.
pub(super) fn resolve_annotation<'a>(
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
pub(super) fn resolve_annotation_trait<'a>(
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
pub(super) fn push_resolved_symbol(
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

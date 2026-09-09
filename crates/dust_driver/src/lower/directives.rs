//! Lowering for library, import, export and part directives.

use super::*;

/// Collects import URIs for backwards-compatible plugin input.
pub(super) fn library_imports(directives: &[ParsedDirective]) -> Vec<String> {
    directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Import { uri, .. } => Some(uri.clone()),
            _ => None,
        })
        .collect()
}

/// Lowers the Dart `library` directive, if present.
pub(super) fn lower_library_directive(
    file_id: dust_text::FileId,
    directives: &[ParsedDirective],
) -> Option<LibraryDeclIr> {
    directives.iter().find_map(|directive| match directive {
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
pub(super) fn lower_library_annotations(
    file_id: dust_text::FileId,
    directives: &[ParsedDirective],
    catalog: &SymbolCatalog,
) -> Vec<AnnotationIr> {
    directives
        .iter()
        .find_map(|directive| match directive {
            ParsedDirective::Library { annotations, .. } => Some(annotations),
            _ => None,
        })
        .into_iter()
        .flatten()
        .map(|annotation| lower_annotation_ir(file_id, annotation, catalog))
        .collect()
}

/// Lowers Dart import directives including combinators and deferred prefixes.
pub(super) fn lower_import_directives(
    file_id: dust_text::FileId,
    directives: &[ParsedDirective],
) -> Vec<ImportIr> {
    directives
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
pub(super) fn lower_export_directives(
    file_id: dust_text::FileId,
    directives: &[ParsedDirective],
) -> Vec<ExportIr> {
    directives
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
pub(super) fn lower_part_directives(
    file_id: dust_text::FileId,
    directives: &[ParsedDirective],
) -> Vec<PartIr> {
    directives
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
pub(super) fn lower_part_of_directive(
    file_id: dust_text::FileId,
    directives: &[ParsedDirective],
) -> Option<PartOfIr> {
    directives.iter().find_map(|directive| match directive {
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

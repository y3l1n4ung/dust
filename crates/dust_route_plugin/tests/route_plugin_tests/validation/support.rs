use dust_ir::{DartFileIr, ImportIr};

use crate::support::span;

pub(super) fn diagnostic_messages(diagnostics: &[dust_diagnostics::Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

pub(super) fn add_import(library: &mut DartFileIr, uri: &str, show: &[&str], hide: &[&str]) {
    library.imports.push(uri.to_owned());
    library.import_directives.push(ImportIr {
        uri: uri.to_owned(),
        prefix: None,
        show: show.iter().map(|name| (*name).to_owned()).collect(),
        hide: hide.iter().map(|name| (*name).to_owned()).collect(),
        is_deferred: false,
        span: span(0, 0),
    });
}

use std::{fs, io, path::Path};

use dust_emitter::{WriteResult, emit_library_with_plan, persist_emit_result};
use dust_workspace::SourceLibrary;

use super::ProcessingConfig;

pub(crate) fn emit_or_write_library(
    library: &SourceLibrary,
    lowered_library: &dust_ir::LibraryIr,
    processing: &ProcessingConfig<'_>,
    plan: dust_plugin_api::SymbolPlan,
) -> io::Result<WriteResult> {
    let previous = read_previous_output(&library.output_path, processing.write_output)?;
    let emitted = emit_library_with_plan(
        lowered_library,
        processing.registry,
        plan,
        previous.as_deref(),
    );

    if processing.write_output {
        persist_emit_result(library.output_path.clone(), emitted)
    } else {
        Ok(WriteResult {
            source: emitted.source,
            symbols: emitted.symbols,
            diagnostics: emitted.diagnostics,
            changed: emitted.changed,
            written: false,
            output_path: library.output_path.clone(),
        })
    }
}

fn read_previous_output(path: &Path, strict: bool) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) if strict => Err(error),
        Err(_) => Ok(None),
    }
}

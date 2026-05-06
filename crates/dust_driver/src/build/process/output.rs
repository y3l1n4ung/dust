use std::{fs, io, path::Path};

use dust_emitter::{
    AuxiliaryWriteResult, WriteResult, emit_library_with_plan, persist_emit_result,
};
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
        let auxiliary_outputs = emitted
            .auxiliary_outputs
            .into_iter()
            .map(|output| {
                let previous = read_previous_output(&output.output_path, false)?;
                Ok(AuxiliaryWriteResult {
                    changed: previous.as_deref() != Some(output.source.as_str()),
                    written: false,
                    output_path: output.output_path,
                    source: output.source,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let changed = emitted.changed || auxiliary_outputs.iter().any(|output| output.changed);

        Ok(WriteResult {
            source: emitted.source,
            symbols: emitted.symbols,
            diagnostics: emitted.diagnostics,
            changed,
            written: false,
            output_path: library.output_path.clone(),
            auxiliary_outputs,
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

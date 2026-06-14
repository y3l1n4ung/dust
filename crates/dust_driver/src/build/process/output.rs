use std::{fs, io, path::Path};

use dust_emitter::{
    AuxiliaryWriteResult, EmitResult, WriteResult, emit_library_with_plan, persist_emit_result,
};
use dust_workspace::SourceLibrary;

use super::ProcessingConfig;

pub(crate) fn emit_or_write_library(
    library: &SourceLibrary,
    lowered_library: &dust_ir::DartFileIr,
    previous_output_hash: Option<Option<u64>>,
    processing: &ProcessingConfig<'_>,
    plan: dust_plugin_api::SymbolPlan,
) -> io::Result<WriteResult> {
    if let Some(Some(previous_hash)) = previous_output_hash {
        let emitted =
            emit_library_with_plan(lowered_library, processing.registry, plan.clone(), None);
        let emitted = if emitted.source.is_empty() && !emitted.changed {
            // Some plugin modes intentionally preserve existing output without
            // contributing source, for example normal builds over DB-only files.
            let previous = read_previous_output(&library.output_path, processing.write_output)?;
            emit_library_with_plan(
                lowered_library,
                processing.registry,
                plan,
                previous.as_deref(),
            )
        } else {
            emitted
        }
        .with_output_hash(&library.output_path);
        return persist_with_previous_hash(
            library.output_path.clone(),
            emitted,
            previous_hash,
            processing.write_output,
        );
    }

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
        let emitted = emitted.with_output_hash(&library.output_path);
        let output_hash = emitted
            .output_hash
            .expect("output hash must be attached before check result assembly");
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
            output_hash,
            symbols: emitted.symbols,
            diagnostics: emitted.diagnostics,
            changed,
            written: false,
            output_path: library.output_path.clone(),
            auxiliary_outputs,
        })
    }
}

fn persist_with_previous_hash(
    output_path: std::path::PathBuf,
    emitted: EmitResult,
    previous_hash: u64,
    write_output: bool,
) -> io::Result<WriteResult> {
    let output_hash = emitted
        .output_hash
        .expect("output hash must be attached before hash-based persistence");
    let has_errors = emitted
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error());
    let changed = output_hash != previous_hash;
    let should_write = write_output && changed && !has_errors;

    if should_write {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, &emitted.source)?;
    }

    let auxiliary_outputs = emitted
        .auxiliary_outputs
        .into_iter()
        .map(|output| {
            if should_write {
                if let Some(parent) = output.output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output.output_path, &output.source)?;
            }
            Ok(AuxiliaryWriteResult {
                source: output.source,
                changed,
                written: should_write,
                output_path: output.output_path,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    Ok(WriteResult {
        source: emitted.source,
        output_hash,
        symbols: emitted.symbols,
        diagnostics: emitted.diagnostics,
        changed,
        written: should_write,
        output_path,
        auxiliary_outputs,
    })
}

fn read_previous_output(path: &Path, strict: bool) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) if strict => Err(error),
        Err(_) => Ok(None),
    }
}

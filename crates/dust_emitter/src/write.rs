use std::{
    fs, io,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::{PluginRegistry, SymbolPlan};

use crate::{emit_library, emit_library_with_plan};

/// The filesystem result of emitting one generated library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteResult {
    /// The emitted `.g.dart` source text.
    pub source: String,
    /// Hash of the emitted primary and auxiliary output path/source pairs.
    pub output_hash: u64,
    /// The reserved generated helper symbols for this file.
    pub symbols: SymbolPlan,
    /// Diagnostics emitted during validation or emission.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether the emitted source differs from the previous output.
    pub changed: bool,
    /// Whether the file was actually written to disk.
    pub written: bool,
    /// The resolved output path used for writing.
    pub output_path: PathBuf,
    /// Additional generated files written for this library.
    pub auxiliary_outputs: Vec<AuxiliaryWriteResult>,
}

/// The filesystem result of writing one auxiliary generated file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxiliaryWriteResult {
    /// The emitted auxiliary source text.
    pub source: String,
    /// Whether the emitted source differs from the previous file contents.
    pub changed: bool,
    /// Whether the auxiliary file was actually written to disk.
    pub written: bool,
    /// The resolved output path used for writing.
    pub output_path: PathBuf,
}

/// Emits and writes one library to its configured output path.
///
/// If error diagnostics are present, the output is not written.
pub fn write_library(library: &DartFileIr, registry: &PluginRegistry) -> io::Result<WriteResult> {
    let output_path = PathBuf::from(&library.output_path);
    let previous_output = read_previous_output(&output_path)?;
    let emitted = emit_library(library, registry, previous_output.as_deref());
    persist_emit_result(output_path, emitted)
}

/// Emits and writes one library using an explicitly prepared symbol plan.
pub fn write_library_with_plan(
    library: &DartFileIr,
    registry: &PluginRegistry,
    plan: SymbolPlan,
) -> io::Result<WriteResult> {
    let output_path = PathBuf::from(&library.output_path);
    let previous_output = read_previous_output(&output_path)?;
    let emitted = emit_library_with_plan(library, registry, plan, previous_output.as_deref());
    persist_emit_result(output_path, emitted)
}

/// Persists one already-emitted generated source to disk.
pub fn persist_emit_result(
    output_path: PathBuf,
    emitted: crate::EmitResult,
) -> io::Result<WriteResult> {
    let emitted = emitted.with_output_hash(&output_path);
    let output_hash = emitted
        .output_hash
        .expect("output hash must be attached before persisting");
    let has_errors = emitted
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error());
    let primary_written = if !has_errors && emitted.changed {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, &emitted.source)?;
        true
    } else {
        false
    };
    let auxiliary_outputs = persist_auxiliary_outputs(&emitted.auxiliary_outputs, has_errors)?;
    let changed = emitted.changed || auxiliary_outputs.iter().any(|output| output.changed);
    let written = primary_written || auxiliary_outputs.iter().any(|output| output.written);

    Ok(WriteResult {
        source: emitted.source,
        output_hash,
        symbols: emitted.symbols,
        diagnostics: emitted.diagnostics,
        changed,
        written,
        output_path,
        auxiliary_outputs,
    })
}

fn read_previous_output(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn persist_auxiliary_outputs(
    outputs: &[crate::emit::AuxiliaryEmitOutput],
    has_errors: bool,
) -> io::Result<Vec<AuxiliaryWriteResult>> {
    outputs
        .iter()
        .map(|output| {
            let previous_output = read_previous_output(&output.output_path)?;
            let changed = previous_output.as_deref() != Some(output.source.as_str());
            let written = if !has_errors && changed {
                if let Some(parent) = output.output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output.output_path, &output.source)?;
                true
            } else {
                false
            };

            Ok(AuxiliaryWriteResult {
                source: output.source.clone(),
                changed,
                written,
                output_path: output.output_path.clone(),
            })
        })
        .collect()
}

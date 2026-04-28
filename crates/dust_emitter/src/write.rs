use std::{
    fs, io,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;
use dust_plugin_api::{PluginRegistry, SymbolPlan};

use crate::{emit_library, emit_library_with_plan};

/// The filesystem result of emitting one generated library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteResult {
    /// The emitted `.g.dart` source text.
    pub source: String,
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
}

/// Emits and writes one library to its configured output path.
///
/// If error diagnostics are present, the output is not written.
pub fn write_library(library: &LibraryIr, registry: &PluginRegistry) -> io::Result<WriteResult> {
    let output_path = PathBuf::from(&library.output_path);
    let previous_output = read_previous_output(&output_path)?;
    let emitted = emit_library(library, registry, previous_output.as_deref());
    finish_write(output_path, emitted)
}

/// Emits and writes one library using an explicitly prepared symbol plan.
pub fn write_library_with_plan(
    library: &LibraryIr,
    registry: &PluginRegistry,
    plan: SymbolPlan,
) -> io::Result<WriteResult> {
    let output_path = PathBuf::from(&library.output_path);
    let previous_output = read_previous_output(&output_path)?;
    let emitted = emit_library_with_plan(library, registry, plan, previous_output.as_deref());
    finish_write(output_path, emitted)
}

fn finish_write(output_path: PathBuf, emitted: crate::EmitResult) -> io::Result<WriteResult> {
    let has_errors = emitted
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error());
    let written = if !has_errors && emitted.changed {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, &emitted.source)?;
        true
    } else {
        false
    };

    Ok(WriteResult {
        source: emitted.source,
        symbols: emitted.symbols,
        diagnostics: emitted.diagnostics,
        changed: emitted.changed,
        written,
        output_path,
    })
}

fn read_previous_output(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

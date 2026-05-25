use std::path::{Path, PathBuf};

use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;
use dust_plugin_api::{AuxiliaryOutputContribution, GENERATED_HEADER, PluginRegistry, SymbolPlan};

use crate::{format::format_generated_source, merge::MergedSections, writer::DartWriter};

/// The in-memory result of emitting one generated library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitResult {
    /// The assembled `.g.dart` source text.
    pub source: String,
    /// Hash of the emitted primary and auxiliary outputs when an output path is known.
    pub output_hash: Option<u64>,
    /// The reserved generated helper symbols for this file.
    pub symbols: SymbolPlan,
    /// Diagnostics emitted during validation or emission.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether the newly emitted source differs from the previous output.
    pub changed: bool,
    /// Additional generated files emitted for this library.
    pub auxiliary_outputs: Vec<AuxiliaryEmitOutput>,
}

/// One in-memory auxiliary output emitted alongside the primary `.g.dart` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxiliaryEmitOutput {
    /// The resolved filesystem output path.
    pub output_path: PathBuf,
    /// The fully rendered source text.
    pub source: String,
}

/// Emits one generated library without touching the filesystem.
pub fn emit_library(
    library: &LibraryIr,
    registry: &PluginRegistry,
    previous_output: Option<&str>,
) -> EmitResult {
    let plan = registry.build_symbol_plan(library);
    emit_library_with_plan(library, registry, plan, previous_output)
}

/// Emits one generated library with an explicitly prepared symbol plan.
pub fn emit_library_with_plan(
    library: &LibraryIr,
    registry: &PluginRegistry,
    plan: SymbolPlan,
    previous_output: Option<&str>,
) -> EmitResult {
    let mut diagnostics = registry.validate_library(library);
    let contributions = registry.emit_contributions(library, &plan);
    diagnostics.extend(
        contributions
            .iter()
            .flat_map(|contribution| contribution.diagnostics.iter().cloned()),
    );
    let merged = MergedSections::from_contributions(&contributions);
    let auxiliary_outputs = collect_auxiliary_outputs(&contributions);
    let (source, changed) = if should_emit_primary(library, &contributions, &plan, &merged) {
        let source = primary_source_override(&contributions)
            .unwrap_or_else(|| assemble_source(library, &plan, &merged));
        let source = format_generated_source(&source);
        let changed = previous_output != Some(source.as_str());
        (source, changed)
    } else {
        (previous_output.unwrap_or_default().to_owned(), false)
    };

    EmitResult {
        source,
        output_hash: None,
        symbols: plan,
        diagnostics,
        changed,
        auxiliary_outputs,
    }
}

impl EmitResult {
    /// Stores the deterministic output-set hash for this emitted result.
    pub fn with_output_hash(mut self, output_path: &Path) -> Self {
        self.output_hash = Some(hash_output_set(
            std::iter::once((output_path, self.source.as_str())).chain(
                self.auxiliary_outputs
                    .iter()
                    .map(|output| (output.output_path.as_path(), output.source.as_str())),
            ),
        ));
        self
    }
}

/// Hashes the generated primary plus auxiliary output path/source pairs.
pub fn hash_output_set<'a>(outputs: impl IntoIterator<Item = (&'a Path, &'a str)>) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for (path, source) in outputs {
        update_hash_bytes(&mut hash, path.to_string_lossy().as_bytes());
        update_hash_bytes(&mut hash, b"\0");
        update_hash_bytes(&mut hash, source.as_bytes());
        update_hash_bytes(&mut hash, b"\0");
    }
    hash
}

fn update_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = (*hash).wrapping_mul(1099511628211);
    }
}

fn should_emit_primary(
    library: &LibraryIr,
    contributions: &[dust_plugin_api::PluginContribution],
    plan: &SymbolPlan,
    merged: &MergedSections,
) -> bool {
    contributions
        .iter()
        .any(|contribution| !contribution.is_empty())
        || !plan.reserved().is_empty()
        || !merged.shared_helpers.is_empty()
        || !merged.support_types.is_empty()
        || !merged.top_level_functions.is_empty()
        || !library_has_dust_symbols(library)
}

fn library_has_dust_symbols(library: &LibraryIr) -> bool {
    library.classes.iter().any(|class| {
        !class.traits.is_empty()
            || !class.configs.is_empty()
            || class.fields.iter().any(|field| field.serde.is_some())
            || class.methods.iter().any(|method| {
                !method.traits.is_empty()
                    || !method.configs.is_empty()
                    || method
                        .params
                        .iter()
                        .any(|param| !param.traits.is_empty() || !param.configs.is_empty())
            })
    }) || library
        .enums
        .iter()
        .any(|enum_ir| !enum_ir.traits.is_empty() || enum_ir.serde.is_some())
}

fn primary_source_override(
    contributions: &[dust_plugin_api::PluginContribution],
) -> Option<String> {
    contributions
        .iter()
        .find_map(|contribution| contribution.primary_source.clone())
}

fn collect_auxiliary_outputs(
    contributions: &[dust_plugin_api::PluginContribution],
) -> Vec<AuxiliaryEmitOutput> {
    contributions
        .iter()
        .flat_map(|contribution| contribution.auxiliary_outputs.iter())
        .map(format_auxiliary_output)
        .collect()
}

fn assemble_source(library: &LibraryIr, plan: &SymbolPlan, merged: &MergedSections) -> String {
    let source_name = Path::new(&library.source_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("source.dart");

    let mut writer = DartWriter::new();
    writer.raw_block(GENERATED_HEADER);
    writer.blank_line();
    writer.line(format!("part of '{source_name}';"));

    let symbol_helpers = render_reserved_helpers(plan);
    if !symbol_helpers.is_empty() {
        writer.blank_line();
        for helper in symbol_helpers {
            writer.line(helper);
        }
    }
    if !merged.shared_helpers.is_empty() {
        writer.blank_line();
        for helper in &merged.shared_helpers {
            writer.raw_block(helper);
        }
    }

    for class in &library.classes {
        let members = merged.members_for_class(&class.name);
        if !members.is_empty() {
            writer.blank_line();
            render_mixin_block(&mut writer, &class.name, members);
        }
    }

    if !merged.support_types.is_empty() {
        writer.blank_line();
        for support in &merged.support_types {
            writer.raw_block(support);
        }
    }
    if !merged.top_level_functions.is_empty() {
        writer.blank_line();
        for function in &merged.top_level_functions {
            writer.raw_block(function);
        }
    }

    writer.finish()
}

fn render_reserved_helpers(plan: &SymbolPlan) -> Vec<String> {
    let mut helpers = Vec::new();

    if plan.contains("_undefined") {
        helpers.push("const Object _undefined = Object();".to_owned());
    }

    helpers
}

fn format_auxiliary_output(output: &AuxiliaryOutputContribution) -> AuxiliaryEmitOutput {
    AuxiliaryEmitOutput {
        output_path: output.output_path.clone(),
        source: format_generated_source(&output.source),
    }
}

fn render_mixin_block(writer: &mut DartWriter, class_name: &str, members: &[String]) {
    let mixin_name = format!("_${class_name}");
    writer.start_block(format!("mixin {mixin_name}"));

    if !members.is_empty() {
        for (index, member) in members.iter().enumerate() {
            if index > 0 {
                writer.blank_line();
            }
            writer.raw_block(member);
        }
    }

    writer.end_block();
}

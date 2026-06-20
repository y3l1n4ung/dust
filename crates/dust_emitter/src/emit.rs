use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
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
    library: &DartFileIr,
    registry: &PluginRegistry,
    previous_output: Option<&str>,
) -> EmitResult {
    let plan = registry.build_symbol_plan(library);
    emit_library_with_plan(library, registry, plan, previous_output)
}

/// Emits one generated library with an explicitly prepared symbol plan.
pub fn emit_library_with_plan(
    library: &DartFileIr,
    registry: &PluginRegistry,
    plan: SymbolPlan,
    previous_output: Option<&str>,
) -> EmitResult {
    let mut diagnostics = registry.validate_library(library);
    let mut contributions = registry.emit_contributions(library, &plan);
    for contribution in &mut contributions {
        diagnostics.append(&mut contribution.diagnostics);
    }
    let primary_source_override = take_primary_source_override(&mut contributions);
    let auxiliary_outputs = collect_auxiliary_outputs(&mut contributions);
    let has_contributions = contributions
        .iter()
        .any(|contribution| !contribution.is_empty());
    let merged = MergedSections::from_contributions(contributions);
    let (source, changed) = if should_emit_primary(
        library,
        has_contributions || primary_source_override.is_some(),
        &plan,
        &merged,
    ) {
        let source =
            primary_source_override.unwrap_or_else(|| assemble_source(library, &plan, &merged));
        let source = format_generated_source(source);
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

/// Updates a stable FNV-1a hash with raw bytes.
fn update_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = (*hash).wrapping_mul(1099511628211);
    }
}

/// Returns true when a primary `.g.dart` file should be emitted.
fn should_emit_primary(
    library: &DartFileIr,
    has_contributions: bool,
    plan: &SymbolPlan,
    merged: &MergedSections,
) -> bool {
    has_contributions
        || !merged.shared_helpers.is_empty()
        || !merged.support_types.is_empty()
        || !merged.top_level_functions.is_empty()
        || !plan.reserved().is_empty()
        || library.classes.iter().any(|class| {
            !merged.members_for_class(&class.name).is_empty()
                || class.fields.iter().any(|field| field.serde.is_some())
        })
        || library.enums.iter().any(|enum_ir| enum_ir.serde.is_some())
}

/// Extracts the first plugin-provided primary source override.
fn take_primary_source_override(
    contributions: &mut [dust_plugin_api::PluginContribution],
) -> Option<String> {
    contributions
        .iter_mut()
        .find_map(|contribution| contribution.primary_source.take())
}

/// Extracts and formats all plugin-provided auxiliary outputs.
fn collect_auxiliary_outputs(
    contributions: &mut [dust_plugin_api::PluginContribution],
) -> Vec<AuxiliaryEmitOutput> {
    contributions
        .iter_mut()
        .flat_map(|contribution| std::mem::take(&mut contribution.auxiliary_outputs))
        .map(format_auxiliary_output)
        .collect()
}

/// Assembles standard Dust generated source from merged plugin sections.
fn assemble_source(library: &DartFileIr, plan: &SymbolPlan, merged: &MergedSections) -> String {
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

/// Renders reserved helper declarations required by the symbol plan.
fn render_reserved_helpers(plan: &SymbolPlan) -> Vec<String> {
    let mut helpers = Vec::new();

    if plan.contains("_undefined") {
        helpers.push("const Object _undefined = Object();".to_owned());
    }

    helpers
}

/// Formats a plugin auxiliary output contribution.
fn format_auxiliary_output(output: AuxiliaryOutputContribution) -> AuxiliaryEmitOutput {
    AuxiliaryEmitOutput {
        output_path: output.output_path,
        source: format_generated_source(output.source),
    }
}

/// Renders the generated mixin block for one source class.
fn render_mixin_block(writer: &mut DartWriter, class_name: &str, members: &[String]) {
    let mixin_name = format!("_${class_name}");
    let mut block = String::with_capacity(
        mixin_name.len() + members.iter().map(String::len).sum::<usize>() + 16,
    );
    writeln!(block, "mixin {mixin_name} {{").expect("writing to String cannot fail");
    for (index, member) in members.iter().enumerate() {
        if index > 0 {
            block.push('\n');
            block.push('\n');
        }
        indent_mixin_member_into(&mut block, member);
    }
    block.push('\n');
    block.push('}');
    writer.raw_block(&block);
}

/// Appends a generated mixin member with two-space indentation.
fn indent_mixin_member_into(out: &mut String, member: &str) {
    for (index, line) in member.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        if line.is_empty() {
            continue;
        }
        out.push_str("  ");
        out.push_str(line);
    }
}

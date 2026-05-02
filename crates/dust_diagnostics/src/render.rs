use std::path::Path;

use dust_text::{FileId, LineCol, LineIndex};

use crate::{Diagnostic, SourceLabel};

/// One source file context used when rendering diagnostics.
#[derive(Debug, Clone, Copy)]
pub struct DiagnosticFileContext<'a> {
    /// The file identifier referenced by diagnostic labels.
    pub file_id: FileId,
    /// The absolute or display-ready source path.
    pub path: &'a Path,
    /// The original UTF-8 source text.
    pub source: &'a str,
    /// The precomputed line index for the source text.
    pub line_index: &'a LineIndex,
}

impl<'a> DiagnosticFileContext<'a> {
    /// Creates one diagnostic file context.
    pub fn new(
        file_id: FileId,
        path: &'a Path,
        source: &'a str,
        line_index: &'a LineIndex,
    ) -> Self {
        Self {
            file_id,
            path,
            source,
            line_index,
        }
    }

    fn line_cols(self, label: &SourceLabel) -> Option<(LineCol, LineCol)> {
        Some((
            self.line_index.line_col(label.range.start())?,
            self.line_index.line_col(label.range.end())?,
        ))
    }

    fn line_text(self, line: u32) -> Option<&'a str> {
        let range = self.line_index.line_range(line as usize)?;
        self.source
            .get(range.start().to_usize()..range.end().to_usize())
    }
}

/// Renders a diagnostic into a stable plain-text form.
///
/// This renderer is intentionally simple at the low-level crate stage.
/// Higher layers can later add richer terminal styling without changing
/// the structured diagnostic data model.
pub fn render_to_string(diagnostic: &Diagnostic) -> String {
    render_with_contexts(diagnostic, &[])
}

/// Renders a diagnostic using optional file contexts for richer label output.
pub fn render_to_string_with_files(
    diagnostic: &Diagnostic,
    files: &[DiagnosticFileContext<'_>],
) -> String {
    render_with_contexts(diagnostic, files)
}

fn render_with_contexts(diagnostic: &Diagnostic, files: &[DiagnosticFileContext<'_>]) -> String {
    let mut output = format!("{}: {}", diagnostic.severity.as_str(), diagnostic.message);

    for label in &diagnostic.labels {
        output.push('\n');
        output.push_str(&render_label(label, files));
    }

    for note in &diagnostic.notes {
        output.push_str(&format!("\n  = note: {note}"));
    }

    output
}

fn render_label(label: &SourceLabel, files: &[DiagnosticFileContext<'_>]) -> String {
    let Some(file) = files
        .iter()
        .copied()
        .find(|file| file.file_id == label.file_id)
    else {
        return render_label_fallback(None, label);
    };

    let Some((start, end)) = file.line_cols(label) else {
        return render_label_fallback(Some(file.path), label);
    };

    if start.line != end.line {
        return render_label_fallback(Some(file.path), label);
    }

    let Some(line_text) = file.line_text(start.line) else {
        return render_label_fallback(Some(file.path), label);
    };

    let line_number = start.line + 1;
    let column_number = start.column + 1;
    let gutter_width = line_number.to_string().len();
    let gutter_pad = " ".repeat(gutter_width);
    let underline_offset = start.column as usize;
    let underline_len = (end.column.saturating_sub(start.column) as usize).max(1);

    format!(
        "  --> {}:{line_number}:{column_number}\n {gutter_pad} |\n {line_number:>gutter_width$} | {line_text}\n {gutter_pad} | {}{} {}",
        file.path.display(),
        " ".repeat(underline_offset),
        "^".repeat(underline_len),
        label.message,
    )
}

fn render_label_fallback(path: Option<&Path>, label: &SourceLabel) -> String {
    match path {
        Some(path) => format!(
            "  --> {} {}..{}: {}",
            path.display(),
            label.range.start().to_u32(),
            label.range.end().to_u32(),
            label.message
        ),
        None => format!(
            "  --> file {:?} {}..{}: {}",
            label.file_id,
            label.range.start().to_u32(),
            label.range.end().to_u32(),
            label.message
        ),
    }
}

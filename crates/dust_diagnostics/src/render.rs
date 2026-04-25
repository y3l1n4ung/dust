use crate::Diagnostic;

/// Renders a diagnostic into a stable plain-text form.
///
/// This renderer is intentionally simple at the low-level crate stage.
/// Higher layers can later add richer terminal styling without changing
/// the structured diagnostic data model.
pub fn render_to_string(diagnostic: &Diagnostic) -> String {
    let mut output = format!("{}: {}", diagnostic.severity.as_str(), diagnostic.message);

    for label in &diagnostic.labels {
        output.push_str(&format!(
            "\n  --> file {:?} {}..{}: {}",
            label.file_id,
            label.range.start().to_u32(),
            label.range.end().to_u32(),
            label.message
        ));
    }

    for note in &diagnostic.notes {
        output.push_str(&format!("\n  = note: {note}"));
    }

    output
}

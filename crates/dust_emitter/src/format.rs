/// Normalizes assembled generated source into a stable final text form.
pub(crate) fn format_generated_source(source: &str) -> String {
    let trimmed = source.trim_end_matches('\n');
    format!("{trimmed}\n")
}

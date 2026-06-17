/// Normalizes assembled generated source into a stable final text form.
pub(crate) fn format_generated_source(mut source: String) -> String {
    while source.ends_with('\n') {
        source.pop();
    }
    source.push('\n');
    source
}

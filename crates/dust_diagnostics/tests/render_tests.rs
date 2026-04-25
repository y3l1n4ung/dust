use dust_diagnostics::{Diagnostic, Severity, SourceLabel, render_to_string};
use dust_text::{FileId, TextRange};

#[test]
fn severity_reports_stable_text() {
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::Warning.as_str(), "warning");
    assert_eq!(Severity::Note.as_str(), "note");
    assert!(Severity::Error.is_error());
    assert!(!Severity::Warning.is_error());
}

#[test]
fn diagnostic_builders_attach_labels_and_notes() {
    let diagnostic = Diagnostic::error("missing semicolon")
        .with_label(SourceLabel::new(
            FileId::new(1),
            TextRange::new(12_u32, 16_u32),
            "field declaration ends here",
        ))
        .with_note("generated output was not written");

    assert!(diagnostic.is_error());
    assert!(diagnostic.has_labels());
    assert_eq!(diagnostic.labels.len(), 1);
    assert_eq!(diagnostic.notes.len(), 1);
}

#[test]
fn render_to_string_includes_labels_and_notes() {
    let diagnostic = Diagnostic::warning("unknown derive trait")
        .with_label(SourceLabel::new(
            FileId::new(4),
            TextRange::new(22_u32, 27_u32),
            "this annotation name is not registered",
        ))
        .with_note("register the symbol in the plugin catalog");

    let rendered = render_to_string(&diagnostic);

    assert!(rendered.contains("warning: unknown derive trait"));
    assert!(rendered.contains("file FileId(4) 22..27"));
    assert!(rendered.contains("this annotation name is not registered"));
    assert!(rendered.contains("register the symbol in the plugin catalog"));
}

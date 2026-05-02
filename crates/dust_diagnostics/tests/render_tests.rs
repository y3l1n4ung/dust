use std::path::PathBuf;

use dust_diagnostics::{
    Diagnostic, DiagnosticFileContext, Severity, SourceLabel, render_to_string,
    render_to_string_with_files,
};
use dust_text::{FileId, LineIndex, TextRange};

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

#[test]
fn contextual_render_includes_path_location_snippet_and_caret() {
    let path = PathBuf::from("/tmp/example/user.dart");
    let source = "@Derive([ToString(), UnknownTrait()])\n";
    let line_index = LineIndex::new(source);
    let diagnostic = Diagnostic::warning("unknown derive trait")
        .with_label(SourceLabel::new(
            FileId::new(4),
            TextRange::new(22_u32, 27_u32),
            "this annotation name is not registered",
        ))
        .with_note("register the symbol in the plugin catalog");

    let rendered = render_to_string_with_files(
        &diagnostic,
        &[DiagnosticFileContext::new(
            FileId::new(4),
            &path,
            source,
            &line_index,
        )],
    );

    assert!(rendered.contains("warning: unknown derive trait"));
    assert!(rendered.contains("  --> /tmp/example/user.dart:1:23"));
    assert!(rendered.contains("1 | @Derive([ToString(), UnknownTrait()])"));
    assert!(rendered.contains("^^^^^ this annotation name is not registered"));
    assert!(rendered.contains("= note: register the symbol in the plugin catalog"));
}

#[test]
fn contextual_render_falls_back_for_multiline_ranges() {
    let path = PathBuf::from("/tmp/example/user.dart");
    let source = "class User {\n  final String name\n}\n";
    let line_index = LineIndex::new(source);
    let diagnostic =
        Diagnostic::error("expected `;` after field declaration").with_label(SourceLabel::new(
            FileId::new(1),
            TextRange::new(12_u32, 33_u32),
            "field spans lines",
        ));

    let rendered = render_to_string_with_files(
        &diagnostic,
        &[DiagnosticFileContext::new(
            FileId::new(1),
            &path,
            source,
            &line_index,
        )],
    );

    assert!(rendered.contains("error: expected `;` after field declaration"));
    assert!(rendered.contains("  --> /tmp/example/user.dart 12..33: field spans lines"));
    assert!(!rendered.contains(" | "));
}

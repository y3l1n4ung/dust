use dust_diagnostics::{Diagnostic, SourceLabel, render_to_string};
use dust_text::{FileId, TextRange};

#[test]
fn renders_missing_semicolon_diagnostic_for_real_dart_field() {
    let dart = "class User {\n  final String name\n}\n";
    let missing_field = dart.find("name").unwrap() as u32;
    let range = TextRange::new(missing_field, missing_field + 4);

    let diagnostic = Diagnostic::error("expected `;` after field declaration").with_label(
        SourceLabel::new(FileId::new(1), range, "field declaration ends here"),
    );

    let rendered = render_to_string(&diagnostic);

    assert!(rendered.contains("error: expected `;` after field declaration"));
    assert!(rendered.contains("field declaration ends here"));
}

#[test]
fn renders_unknown_annotation_diagnostic_for_real_dart_metadata() {
    let dart = "@UnknownTrait()\nclass User {}\n";
    let start = dart.find("UnknownTrait").unwrap() as u32;
    let end = start + "UnknownTrait".len() as u32;

    let diagnostic = Diagnostic::warning("unknown derive trait or config `UnknownTrait`")
        .with_label(SourceLabel::new(
            FileId::new(2),
            TextRange::new(start, end),
            "annotation is not owned by any registered plugin",
        ))
        .with_note("known traits are resolved through plugin-owned symbol catalogs");

    let rendered = render_to_string(&diagnostic);

    assert!(rendered.contains("warning: unknown derive trait or config `UnknownTrait`"));
    assert!(rendered.contains("annotation is not owned by any registered plugin"));
    assert!(rendered.contains("plugin-owned symbol catalogs"));
}

use dust_text::{FileId, SourceText, TextRange, TextSize};

fn sample_source() -> SourceText {
    SourceText::new(
        FileId::new(1),
        "import 'dart:convert';\npart 'user.g.dart';\n\n@Derive([Debug(), Eq()])\nclass User {\n  final String? nickname;\n}\n",
    )
}

#[test]
fn slices_part_directive_from_real_dart_source() {
    let source = sample_source();
    let range = TextRange::new(23_u32, 42_u32);

    assert_eq!(source.slice(range), Some("part 'user.g.dart';"));
}

#[test]
fn slices_annotation_line_from_real_dart_source() {
    let source = sample_source();
    let range = TextRange::new(44_u32, 68_u32);

    assert_eq!(source.slice(range), Some("@Derive([Debug(), Eq()])"));
}

#[test]
fn reports_line_and_column_for_nullable_field() {
    let source = sample_source();
    let offset = TextSize::new(98);
    let line_col = source.line_col(offset).unwrap();

    assert_eq!(line_col.line, 5);
    assert_eq!(line_col.column, 16);
}

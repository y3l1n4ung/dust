use dust_text::{FileId, LineIndex, SourceText, TextRange, TextSize};

#[test]
fn text_size_supports_arithmetic() {
    let start = TextSize::new(5);
    let width = TextSize::new(7);

    assert_eq!((start + width).to_u32(), 12);
    assert_eq!((start + width - start).to_u32(), 7);
}

#[test]
fn text_range_reports_length_and_membership() {
    let range = TextRange::new(3_u32, 9_u32);

    assert_eq!(range.start().to_u32(), 3);
    assert_eq!(range.end().to_u32(), 9);
    assert_eq!(range.len().to_u32(), 6);
    assert!(!range.is_empty());
    assert!(range.contains(3_u32));
    assert!(range.contains(8_u32));
    assert!(!range.contains(9_u32));
}

#[test]
fn text_range_can_cover_other_ranges() {
    let left = TextRange::new(4_u32, 10_u32);
    let right = TextRange::new(8_u32, 14_u32);

    assert!(left.intersects(right));
    assert_eq!(left.cover(right), TextRange::new(4_u32, 14_u32));
}

#[test]
fn line_index_tracks_offsets() {
    let index = LineIndex::new("first\nsecond\nthird");

    assert_eq!(index.line_count(), 3);
    assert_eq!(index.line_start(0), Some(TextSize::new(0)));
    assert_eq!(index.line_start(1), Some(TextSize::new(6)));
    assert_eq!(index.line_col(TextSize::new(8)).unwrap().line, 1);
    assert_eq!(index.line_col(TextSize::new(8)).unwrap().column, 2);
}

#[test]
fn source_text_slices_and_reports_positions() {
    let source = SourceText::new(FileId::new(7), "class User {\n  final String name;\n}\n");
    let field_range = TextRange::new(15_u32, 32_u32);

    assert_eq!(source.file_id(), FileId::new(7));
    assert_eq!(source.slice(field_range), Some("final String name"));
    assert_eq!(source.line_col(TextSize::new(15)).unwrap().line, 1);
}

use dust_parser_dart::{ParseBackend, ParseOptions, ParseResult};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_text::{FileId, SourceText};

pub(crate) fn parse(file_id: u32, source: &str) -> ParseResult {
    let source = SourceText::new(FileId::new(file_id), source);
    TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default())
}

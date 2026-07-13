//! Integration tests for resolver-normalized HTTP configuration.

use dust_ir::{HttpConfigIr, HttpParseThreadIr, HttpTargetIr, NormalizedConfigIr};
use dust_parser_dart::{ParseBackend, ParseOptions};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{SymbolCatalog, resolve_library};
use dust_text::{FileId, SourceText};

#[test]
fn normalizes_http_client_options_from_parser_owned_values() {
    let source = SourceText::new(
        FileId::new(91),
        r#"
part 'api.g.dart';

@HttpClient(
  baseUrl: 'https://api.example.test',
  target: HttpTarget.flutter,
  parseThread: HttpParseThread.isolate,
  headers: {'x-client': 'dust'},
  generateTest: true,
)
class Api {}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let mut catalog = SymbolCatalog::new();
    catalog.register_config("HttpClient", "dust_dart::HttpClient");
    let resolved = resolve_library(
        FileId::new(91),
        "lib/api.dart",
        "lib/api.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let Some(NormalizedConfigIr::Http(HttpConfigIr::Client(client))) =
        resolved.library.classes[0].configs[0].normalized.as_ref()
    else {
        panic!("expected normalized HttpClient configuration");
    };
    assert_eq!(client.base_url.as_deref(), Some("https://api.example.test"));
    assert_eq!(client.target, HttpTargetIr::Flutter);
    assert_eq!(client.parse_thread, HttpParseThreadIr::Isolate);
    assert_eq!(client.headers, [("x-client".to_owned(), "dust".to_owned())]);
    assert!(client.generate_test);
}

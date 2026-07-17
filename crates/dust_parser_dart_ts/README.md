# dust_parser_dart_ts

Tree-sitter-backed Dart syntax extraction for Dust.

This crate implements the `dust_parser_dart` backend interface. It reads Dart
source files and extracts parser facts used by the resolver, IR lowering, and
plugins.

## Owns

- Tree-sitter parser setup for Dart
- syntax error recovery and parser diagnostics
- extraction of classes, enums, directives, annotations, constructors, fields,
  methods, and query helper calls
- preserved source spans for diagnostics and generated-code mapping
- parser fixtures for supported Dart syntax

Tree-sitter nodes and cursors stay private to this crate. Callers receive Dust
parser surfaces and lowered data, not raw Tree-sitter handles.

## Design Rules

- Extract syntax only; do not perform semantic analysis here.
- Preserve enough source text and spans for later diagnostics.
- Prefer parser facts over plugin-local source-string parsing.
- Add fixtures before relying on newly supported Dart syntax.

Use `dust_parser_dart` for backend-neutral parser contracts and `dust_ir` for
semantic data consumed by plugins.

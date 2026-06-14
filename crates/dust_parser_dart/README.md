# dust_parser_dart

Backend-neutral parser contracts and extracted Dart file surfaces.

## Owns

- `ParseBackend`
- `ParseOptions`
- `ParseResult`
- `ParsedDartFileSurface` and temporary `ParsedLibrarySurface` compatibility alias
- parsed directive, class, enum, field, method, constructor, annotation, and query-call facts

## Used by

- `dust_parser_dart_ts`
- `dust_plugin_api`
- `dust_resolver`

## Edit here when

- the parse surface contract changes
- parser-backed IR or resolver needs more source detail
- backend-independent parsing behavior changes

Tree-sitter types must not appear in this crate's public API. The concrete
tree-sitter backend owns syntax tree traversal and returns Dust-owned facts.

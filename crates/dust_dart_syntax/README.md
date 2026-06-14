# dust_dart_syntax

Shared Dart source parsing helpers for Dust crates.

## Owns

- top-level comma splitting for annotation/list argument snippets
- balanced parenthesis detection for preserved Dart expressions
- simple literal parsing for strings, booleans, type/member references, and maps

## Used by

- `dust_parser_dart`
- `dust_parser_dart_ts`
- `dust_driver`
- `dust_dart_emit` through compatibility re-exports

## Edit here when

- a parser, resolver, or plugin needs the same Dart snippet helper in more than
  one crate
- a preserved expression parser needs stricter delimiter or literal handling
- source-audit tests identify duplicated ad hoc Dart parsing

Public APIs in this crate are documented and compiled with
`#![deny(missing_docs)]`. Runtime behavior is covered by the `dust_dart`
helper coverage gate in [../../docs/developer.md](../../docs/developer.md).

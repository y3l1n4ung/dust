# dust_text

Low-level source text primitives for Dust.

## Owns

- `FileId`
- `SourceText`
- line/column indexing
- text ranges and sizes

## Used by

- `dust_parser_dart`
- `dust_diagnostics`
- `dust_ir`
- `dust_resolver`

## Edit here when

- parser or diagnostics need new source slicing behavior
- range math or line indexing changes
- source text ownership or sharing strategy changes

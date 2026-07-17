# dust_text

Text and span primitives shared by Dust parser, diagnostics, and IR crates.

This crate provides the small types used to refer to source files and source
ranges without passing raw paths and integer offsets through the whole pipeline.

## Owns

- `SourceText`
- `TextRange`
- `TextSize`
- `FileId`
- line and column lookup helpers
- range-safe source slicing

## Design Rules

- Keep byte offsets explicit.
- Preserve UTF-8 boundary correctness.
- Keep spans compact enough to move through parser and IR structures.
- Avoid feature-specific logic here.

Use this crate when a lower-level crate needs stable source locations or
diagnostic ranges.

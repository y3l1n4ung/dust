# dust_resolver

Semantic resolution layer for Dust.

This crate enriches parser facts with imports, symbols, annotation identities,
constructor data, and normalized feature configuration before plugin validation
and generation.

## Owns

- Dust symbol catalog
- annotation identity resolution
- import and part resolution
- type and constructor normalization
- resolver diagnostics
- feature diagnostic scope derived from normalized config
- conversion support for canonical `DartFileIr`

## Design Rules

- Unknown symbols should produce diagnostics, not panics.
- Keep compatibility fallbacks explicit when short annotation names are accepted.
- Prefer structured annotation values over raw string parsing.
- Preserve spans for diagnostics.
- Keep plugin-owned behavior out of the resolver.

Resolver output feeds driver lowering and plugin validation. It should answer
what the source means to Dust without deciding what code a feature emits.

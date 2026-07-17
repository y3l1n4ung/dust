# dust_driver

Build, check, watch, clean, doctor, i18n, and DB orchestration for Dust.

This crate connects workspace discovery, parsing, resolution, plugin execution,
emission, and cache updates behind the CLI commands exposed by `dust_cli`.

## Pipeline

1. Scan package roots and Dart source files.
2. Parse files into Dust-owned syntax facts.
3. Resolve imports, symbols, types, annotations, and feature config.
4. Run plugin validation and generation.
5. Assemble deterministic `.g.dart` output.
6. Write changed files and update cache metadata.

## Owns

- build/check/watch scheduling
- worker coordination and `--jobs`
- `--fail-fast` behavior
- stale-file detection for `dust check`
- cache invalidation rules for cross-file features
- i18n and DB command orchestration
- progress summaries and build result counts

## Edit Here When

- command behavior needs driver support
- generated file discovery or write behavior changes
- cache invalidation changes
- a plugin needs workspace-level facts
- watch mode, worker scheduling, or progress reporting changes

The CLI owns argument parsing and terminal UX. Plugins own feature-specific
generation.

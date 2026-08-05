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

## Validating Plugin Modes

Focused validation commands select a plugin by its stable name and pass a
shared execution mode describing online/offline access and metadata writes.
Other plugins register only their symbol claims, so their annotations remain
resolvable without running their validation or generation.

The driver owns profile selection and deterministic cache salts. The selected
plugin owns the meaning of validation and its metadata. For example, the
Database plugin continues to own SQLx connections, query validation, and query
cache files.

## Owns

- build/check/watch scheduling
- worker coordination and `--jobs`
- `--fail-fast` behavior
- stale-file detection for `dust check`
- cache invalidation rules for cross-file features
- i18n and DB command orchestration
- progress summaries and build result counts

## Does Not Own

- feature-specific annotation meaning
- feature validation rules
- generated Dart feature behavior

## Edit Here When

- command behavior needs driver support
- generated file discovery or write behavior changes
- cache invalidation changes
- a plugin needs workspace-level facts
- watch mode, worker scheduling, or progress reporting changes

The CLI owns argument parsing and terminal UX. Plugins own feature-specific
generation.

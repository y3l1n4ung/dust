# Parser Architecture Refactor

Tracking issue: [#43](https://github.com/y3l1n4ung/dust/issues/43)

## Goal

Make `DartFileIr` the single parser/codegen model after tree-sitter parsing.
Tree-sitter stays private to `dust_parser_dart_ts`, resolver enriches the file
model, plugins consume normalized IR only, and generated Dart behavior remains
stable unless a slice explicitly adds parser support.

## Current State

- `DartFileIr` exists in `dust_ir`; `LibraryIr` is a temporary compatibility
  alias for older tests and external callers.
- `ParsedDartFileSurface` exists in `dust_parser_dart`; `ParsedLibrarySurface`
  is a temporary compatibility alias.
- Structured annotation arguments are extracted by the tree-sitter backend and
  parser accessors prefer those facts before falling back to raw source.
- Parser directive extraction preserves `library`, import prefixes, exports,
  parts, and `part of` names; driver lowering carries those facts into
  `DartFileIr`.
- Parser type extraction preserves exact type source plus normalized type shape
  for fields, method returns, method parameters, and constructor parameters;
  legacy `type_source` fields are compatibility mirrors of parser-owned type
  facts, and driver lowering prefers those facts before raw-source fallback.
- Parser declaration name extraction uses grammar fields or direct children for
  fields and formal parameters; source audits block broad last-descendant
  identifier guessing in class extraction.
- Parser class modifier extraction uses tree-sitter keyword tokens for
  `abstract`, `interface`, and `mixin class`; source audits block header-source
  parsing for class modifiers.
- Parser method modifier and body extraction uses tree-sitter keyword tokens and
  `function_body` nodes; source audits block method header/body source parsing.
- Parser constructor extraction uses grammar fields for constructor names and
  redirecting factory targets; source audits block target parsing via `=`
  splitting or character scanning.
- Parser field initializer detection and formal-parameter default extraction use
  tree-sitter value/expression nodes; source audits block default-value source
  scanners and initializer checks based on `contains('=')`.
- DB query helper discovery is tree-sitter-backed, including selector-chain
  calls in block statements and chained fetch methods.
- Parser extraction now carries all canonical `DartFileIr` declaration buckets:
  classes, enums, mixins, extensions, extension types, top-level functions,
  top-level variables, typedefs, directives, and DB query calls. Driver lowering
  fills those additive IR fields while keeping unsupported declarations inert
  for current plugins.
- Resolved config applications now preserve parser-owned structured annotation
  argument values alongside the temporary raw `arguments_source` compatibility
  field. Existing config accessors read structured expression values first and
  fall back to raw source, giving plugins a normalized migration path without
  changing generated Dart output.
- Production Rust source uses `DartFileIr` and `ParsedDartFileSurface`; source
  audits block new compatibility-name usage outside shim modules.
- The plugin API has `PluginContext`, `GeneratedUnit`, and `generate`, with
  `emit` still acting as the compatibility path for existing plugins.

## Implementation Plan

- [x] Establish canonical file/model names and compatibility aliases.
- [x] [#44](https://github.com/y3l1n4ung/dust/issues/44) Expand `DartFileIr` to carry directives, annotations, class-like
  declarations, top-level declarations, query calls, and preserved expression
  sources.
- [x] [#45](https://github.com/y3l1n4ung/dust/issues/45) Move annotation, type, declaration, and DB query lowering behind
  tree-sitter extraction helpers. Directive and DB query lowering are now
  parser-backed; field, method, and constructor type lowering now prefers
  parser-owned type facts and no longer uses declaration-prefix type parsing.
  Field and formal-parameter names now use grammar fields/direct children.
  Class/method modifiers, method bodies, constructor names, redirecting factory
  targets, field initializer flags, and formal-parameter defaults now use
  grammar fields, keyword tokens, body nodes, or expression nodes. Remaining
  top-level declaration buckets now have parser surfaces and additive
  `DartFileIr` lowering.
- [x] [#47](https://github.com/y3l1n4ung/dust/issues/47) Replace DB query helper source scanning with tree-sitter call-expression extraction.
- [ ] [#48](https://github.com/y3l1n4ung/dust/issues/48) Make resolver consume and return enriched `DartFileIr` with resolved
  annotation/type symbols and normalized feature configs. Initial resolver
  enrichment now carries structured config annotation arguments in
  `ConfigApplicationIr`; full `DartFileIr` return and feature-specific config
  normalization remain.
- [ ] [#46](https://github.com/y3l1n4ung/dust/issues/46) Migrate plugins from parser-surface/raw-source parsing to normalized IR.
- [ ] [#49](https://github.com/y3l1n4ung/dust/issues/49) Remove compatibility aliases once production crates and tests have moved.
- [ ] [#49](https://github.com/y3l1n4ung/dust/issues/49) Harden audits so parser/source parsing helpers cannot spread back into
  driver or plugin code.

## Tests

- Parser golden snapshots for full `DartFileIr` fixtures.
- Syntax fixtures for annotations, prefixed annotations, directives,
  constructors, fields, methods, enums, mixins, extensions, extension types,
  top-level declarations, type shapes, and DB query calls.
- Dart-version compatibility fixtures for supported grammar versions.
- Per-slice Rust gates: `rtk cargo fmt --all -- --check`, targeted
  `rtk cargo test -p ...`, `rtk cargo clippy --workspace --all-targets -- -D warnings`,
  and `rtk git diff --check`.

## Release Criteria

- `tree_sitter::Node` is not exposed outside `dust_parser_dart_ts`.
- Plugins validate and generate from `DartFileIr` and normalized config/type
  data.
- `LibraryIr` and `ParsedLibrarySurface` aliases are removed.
- Existing generated Dart output remains deterministic and analyzer-clean.

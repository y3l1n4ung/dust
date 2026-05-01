# Developer Guide

This document describes the Dust workspace layout, the current build pipeline,
and the rules new annotations must follow.

For the plugin-authoring workflow, see [plugin-guide.md](plugin-guide.md).

## Workspace Layout

### Rust crates

| Crate | Role |
| --- | --- |
| `dust_text` | shared text primitives and source containers |
| `dust_diagnostics` | diagnostics model and formatting |
| `dust_cache` | persistent `.dart_tool/dust` cache storage |
| `dust_parser_dart` | parser contract and extracted source surface types |
| `dust_parser_dart_ts` | tree-sitter Dart backend |
| `dust_ir` | lowered language-neutral IR used by plugins and emitter |
| `dust_resolver` | type, annotation, and symbol resolution |
| `dust_plugin_api` | plugin registration, symbol plans, shared workspace analysis |
| `dust_plugin_derive` | `ToString`, `Eq`, and `CopyWith` generation |
| `dust_plugin_serde` | JSON encode/decode generation and codecs |
| `dust_emitter` | `.g.dart` assembly and write path |
| `dust_workspace` | package discovery and package-config resolution |
| `dust_driver` | build, check, watch, clean, and doctor orchestration |
| `dust_cli` | the `dust` command-line interface |

### Dart packages

| Package | Role |
| --- | --- |
| `derive_annotation` | derive annotations for core codegen |
| `derive_serde_annotation` | serde annotations and codec contract |

### Example projects

| Project | Role |
| --- | --- |
| `examples/product_showcase` | real example package with analyzer and runtime tests |
| `examples/stress_project` | generated mixed corpus for scale and perf validation |

## Build Pipeline

Every annotation must use the same five-stage pipeline.

1. Workspace discovery
   - `dust_workspace` resolves the package root, cache root, and package config.
2. Source load and cache check
   - `dust_driver` reads source, hashes source/output/config/tool state, and
     decides hit vs miss.
3. Shared analysis scan
   - missed files are parsed once
   - plugins collect parse-only cross-file facts into
     `WorkspaceAnalysisBuilder`
   - per-library analysis snapshots are cached
4. Resolve and lower
   - `dust_resolver` resolves the parsed surface
   - `dust_driver::lower` converts resolved data into Dust IR
5. Emit and write
   - plugins contribute symbols and generated members
   - `dust_emitter` assembles and writes the final `.g.dart`

## Crate Connection Map

Use this when deciding where a change belongs.

1. Parsing a new Dart syntax form
   - edit `dust_parser_dart_ts`
   - extend `dust_parser_dart` surfaces only if the contract itself changes
2. Understanding a new annotation or constructor rule
   - edit `dust_resolver`
   - lower new resolved data into `dust_ir` only if plugins need normalized data
3. Adding normalized generator inputs
   - edit `dust_ir`
   - update `dust_driver::lower`
4. Adding cross-file facts
   - edit `dust_plugin_api` shared analysis contracts
   - collect facts in the plugin during the parse-only scan phase
   - do not add plugin-specific branches in `dust_driver`
5. Changing generated Dart output
   - edit the owning plugin and sometimes `dust_emitter`
6. Changing cache or scheduling behavior
   - edit `dust_driver` and `dust_cache`
7. Changing package discovery or pub workspace behavior
   - edit `dust_workspace`
8. Changing command UX
   - edit `dust_cli`

See [../crates/README.md](../crates/README.md) and each crate README for the
short per-crate ownership guide.

## Shared Analysis Rules

Shared analysis exists so future annotations follow the same fast path as
`CopyWith`.

- Plugins may collect workspace facts only from parsed surfaces.
- Cross-file facts should be stored in `WorkspaceAnalysisBuilder`.
- Cached files must still contribute their saved analysis snapshots.
- Emission must read workspace facts from `SymbolPlan`, not by rescanning the
  workspace.

Today this path is used by derive for cross-file `CopyWith` support. Future
plugins should extend the same mechanism instead of adding driver branches.

## Performance Rules

- Source text should be loaded once and shared across scan and build work.
- Avoid cloning large workspace facts into every file plan.
- Cache data should stay package-local under `.dart_tool/dust`.
- Perf work must be validated against the mixed 5k stress corpus, not a
  single-feature fixture.
- Prefer parse-time collection for workspace facts and emit-time reuse of
  immutable shared analysis.

## Adding A New Annotation

1. Add the public Dart annotation API in the right package.
2. Extend resolver or lowering only where the new surface needs IR support.
3. Implement a plugin contribution in the Rust plugin crate.
4. If the feature needs cross-file knowledge, use shared workspace analysis.
5. Add Rust tests for parsing, lowering, validation, and generation.
6. Add real Dart coverage in `examples/product_showcase`.
7. Extend `examples/stress_project` when the feature affects build throughput
   or shared analysis.
8. Update package docs and the relevant roadmap doc.

## What To Edit When

| Change | Primary crates |
| --- | --- |
| new Dart syntax support | `dust_parser_dart_ts`, sometimes `dust_parser_dart` |
| new IR field or type shape | `dust_ir`, `dust_driver` lowering |
| new derive-style feature | annotation package, `dust_resolver`, `dust_ir`, plugin crate, showcase |
| new cross-file feature | plugin crate, `dust_plugin_api`, `dust_cache`, `dust_driver` |
| generated file layout/header | `dust_emitter` |
| build/watch/check behavior | `dust_driver`, `dust_cli` |
| package/workspace discovery | `dust_workspace` |
| cache persistence | `dust_cache` |

## Stress Project

`examples/stress_project` intentionally mixes:

- derive-only models
- nested serde models
- serde rename/default/skip/alias cases
- codec-backed fields
- enhanced enums
- cross-file linked models

This keeps perf work honest. New features that affect the general pipeline
should extend the generator matrix rather than adding a one-off benchmark.

Generate and benchmark locally:

```bash
cd examples/stress_project
dart pub get
./generate.sh --count 5000

cd ../..
cargo run -p dust_cli -- build --root examples/stress_project
cargo test -p dust_cli stress_project_release_build_stays_fast -- --ignored --nocapture
```

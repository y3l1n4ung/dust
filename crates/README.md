# Crates

Dust is split into focused Rust crates. The dependency flow is intentionally
one-way:

`text -> diagnostics -> parser -> resolver -> IR -> plugin API/plugins -> emitter -> workspace/driver -> CLI`

## Crate Index

| Crate | Owns | Edit here when |
| --- | --- | --- |
| `dust_text` | source text containers and ranges | you need new text primitives or source indexing |
| `dust_diagnostics` | error and warning structures | you need richer diagnostics or formatting |
| `dust_cache` | persistent `.dart_tool/dust` cache entries | cache schema or persisted fingerprints change |
| `dust_parser_dart` | backend-neutral parse contracts and surfaces | parse surface contracts change |
| `dust_parser_dart_ts` | tree-sitter extraction backend | Dart syntax extraction changes |
| `dust_ir` | lowered Dust IR | generator features need new IR shape |
| `dust_resolver` | annotation, type, and constructor resolution | parsed data needs semantic interpretation |
| `dust_plugin_api` | plugin interfaces and shared workspace analysis | new plugin hooks or shared analysis rules |
| `dust_plugin_derive` | `ToString`, `Eq`, `CopyWith` generation | core derive behavior changes |
| `dust_plugin_serde` | JSON encode/decode generation | serde behavior or validation changes |
| `dust_emitter` | final `.g.dart` assembly and write path | file header, layout, or emission ordering changes |
| `dust_workspace` | package root and package-config discovery | workspace/package resolution changes |
| `dust_driver` | build, check, watch, clean, doctor orchestration | pipeline, cache, scheduling, or progress changes |
| `dust_cli` | `dust` command parsing and terminal output | command surface or UX changes |

## Rules

- New annotations must reuse the same `dust_driver` pipeline.
- Cross-file facts belong in `dust_plugin_api` shared analysis, not driver
  special cases.
- Public behavior changes need Rust tests plus real Dart coverage.
- Perf-sensitive changes should be checked against
  `examples/stress_project`.

See [../docs/developer.md](../docs/developer.md) for the full pipeline and
[../CONTRIBUTING.md](../CONTRIBUTING.md) for contributor workflow.

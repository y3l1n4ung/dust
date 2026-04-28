# Dust Roadmap

This roadmap tracks Dust feature work after the first public release cut. Each
feature must land with annotations, IR support, generator output, analyzer-backed
tests, package docs, and examples.

## Principles

- Generated Dart must be analyzer-clean, deterministic, readable, and small.
- Public APIs must have Dartdoc before release.
- Features should work without `build_runner`.
- Rust crates stay focused: parser, IR, resolver, plugin API, plugin, emitter,
  driver, CLI.
- Every feature needs golden Rust tests plus real Dart analyzer/tests.

## Tracks

| Track | Priority | Plan |
| --- | --- | --- |
| Generated code quality | P0 | [generated-code-quality.md](generated-code-quality.md) |
| Serde | P0 | [serde.md](serde.md) |
| HttpClient | P1 | [http-client.md](http-client.md) |
| Route annotations | P1 | [routing.md](routing.md) |
| State management | P2 | [state-management.md](state-management.md) |

## Release Gate

- `cargo fmt --all --check`
- `cargo test --workspace --quiet`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Dart package `dart analyze` and `dart test`
- Product showcase generation, analyzer, and tests
- CI green on GitHub

# Dust Roadmap

This roadmap tracks Dust feature work after the first public release cut. Each
feature must land with annotations, IR support, generator output, analyzer-backed
tests, package docs, and examples.

## How To Read This Folder

Each roadmap document should answer the same questions:

- What problem the feature solves.
- What public API shape Dust intends to expose.
- What is already working today.
- What is still missing before the feature is release-ready.
- How the feature will be validated in Rust and Dart tests.

## Principles

- Generated Dart must be analyzer-clean, deterministic, readable, and small.
- Public APIs must have Dartdoc before release.
- Features should work without `build_runner`.
- Rust crates stay focused: parser, IR, resolver, plugin API, plugin, emitter,
  driver, CLI.
- Every feature needs golden Rust tests plus real Dart analyzer/tests.

## Tracks

| Track | Priority | Covers | Plan |
| --- | --- | --- | --- |
| Generated code quality | P0 | `ToString()`, `Eq()`, `CopyWith()` output quality | [generated-code-quality.md](generated-code-quality.md) |
| Serde | P0 | JSON encode/decode, rename rules, defaults, custom codecs | [serde.md](serde.md) |
| HttpClient | P1 | Dio-backed API client generation | [http-client.md](http-client.md) |
| Route annotations | P1 | Navigator 2.0 typed routing | [routing.md](routing.md) |
| State management | P2 | Typed state stores and action generation | [state-management.md](state-management.md) |

## Documentation Standard

Roadmap docs should stay concise, but each one should keep these sections when
possible:

- `Goal`
- `Current State`
- `Public API` or `API Sketch`
- `Generator Work` or `Implementation Plan`
- `Tests`
- `Release Criteria` or `Done`
- use `- [ ]` for open work items and `- [x]` for completed milestones when a
  section is action-oriented

## Release Gate

- `cargo fmt --all --check`
- `cargo test --workspace --quiet`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Dart package `dart analyze` and `dart test`
- Product showcase generation, analyzer, and tests
- CI green on GitHub

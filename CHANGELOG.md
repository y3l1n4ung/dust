# Changelog

All notable changes to Dust are documented in this file.

## [0.1.0] - 2026-05-08

### Added

- Rust workspace crates for parsing, resolution, IR lowering, plugin
  execution, emitting, caching, workspace discovery, driver orchestration, and
  the `dust` CLI.
- Derive generation for `ToString`, `Eq`, and `CopyWith`.
- Serde generation for `Serialize`, `Deserialize`, rename rules, defaults, and
  custom field codecs.
- Dio-backed HttpClient generation with request annotations, shared client
  configuration, generated request-mapping tests, and real example APIs.
- Shared output policy with configurable generated suffixes and generated tests
  under `test/generated/..._test.dart`.
- Product showcase and stress-project examples for derive, serde, and HTTP
  generation.
- GitHub release workflow that publishes tagged CLI binaries and checksums.

### Changed

- The HTTP annotation package now re-exports the supported Dio surface and
  `dart:convert` from a single import.
- Generated examples and docs now use the single-import HttpClient package
  surface.
- Release prep now includes a temporary ordered Rust publish helper for the
  first manual crates.io release.

### Notes

- `v0.1.0` is the first public Dust release.
- Manual release notes for GitHub live in `release-notes/v0.1.0.md`.

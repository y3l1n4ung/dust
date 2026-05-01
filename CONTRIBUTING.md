# Contributing

Dust is a Rust workspace plus Dart annotation packages and example projects.
Changes should keep generated Dart small, readable, analyzer-clean, and fast to
build at workspace scale.

## Repository Map

- `crates/`: Rust workspace crates for parsing, IR, resolver, plugin API,
  plugins, emitter, workspace discovery, driver, cache, and CLI.
- `packages/`: publishable Dart annotation packages.
- `examples/product_showcase/`: real Dart example package with analyzer and
  runtime tests.
- `examples/stress_project/`: large generated fixture for scale and perf work.
- `docs/roadmap/`: feature plans and release gates.
- `docs/developer.md`: internal architecture and pipeline guide.

## Setup

```bash
cargo test --workspace --quiet
cargo clippy --workspace --all-targets --all-features -- -D warnings
dart pub get
```

For Dart packages and examples:

```bash
cd packages/derive_annotation && dart analyze && dart test
cd packages/derive_serde_annotation && dart analyze && dart test
cd examples/product_showcase && dart analyze && dart test
```

## Local Build Commands

Generate the showcase:

```bash
cargo run -p dust_cli -- build --root examples/product_showcase
```

Generate the stress project:

```bash
cd examples/stress_project
dart pub get
./generate.sh --count 5000

cd ../..
cargo run -p dust_cli -- build --root examples/stress_project
```

## Change Rules

- Keep one shared build pipeline for all annotations and plugins.
- Do not add plugin-specific fast paths in the driver.
- Reuse workspace analysis for cross-file facts instead of rescanning in emit.
- Keep non-test source files and test modules small and focused.
- Add Dartdoc for public Dart APIs before release.
- Generated output must stay deterministic and analyzer-clean.

## Tests To Add With Feature Work

- Rust unit or integration tests for parser, lowering, plugin, or emitter
  behavior.
- Real Dart analyzer and runtime coverage in `examples/product_showcase/`.
- Stress-project coverage when the feature affects build throughput, cache
  shape, or shared analysis.

## Perf Workflow

Release perf smoke test:

```bash
cargo test -p dust_cli stress_project_release_build_stays_fast -- --ignored --nocapture
```

Optional thresholds:

```bash
DUST_PERF_COLD_MAX_MS=2000 DUST_PERF_WARM_MAX_MS=800 \
  cargo test -p dust_cli stress_project_release_build_stays_fast -- --ignored --nocapture
```

When changing the build path, report both:

- cold build on `examples/stress_project`
- warm cached build on `examples/stress_project`

## Pull Requests

- Use conventional commit prefixes for local commits.
- Include the commands you ran.
- Call out any cache schema change, generated output header change, or public
  Dart API change.

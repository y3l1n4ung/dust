# Developer Guide: Dust Architecture

This document provides a deep dive into the Dust workspace layout, internal build pipeline, and core engineering principles. It is the primary resource for contributors working on the Rust engine or core plugins.

---

## 📂 Workspace Layout

### Rust Engine (`crates/`)

| Crate | Responsibility |
| :--- | :--- |
| `dust_driver` | **The Orchestrator.** Manages the build lifecycle, caching, and worker threads. |
| `dust_parser_dart_ts` | Tree-sitter backend for high-fidelity Dart parsing. Tree-sitter nodes stay private to this crate. |
| `dust_resolver` | Enriches parsed Dart file IR with imports, symbols, annotation identities, and normalized feature config. |
| `dust_ir` | The canonical `DartFileIr` model used as the contract between the engine and plugins. |
| `dust_emitter` | Merges plugin contributions and writes deterministic `.g.dart` files. |
| `dust_diagnostics` | High-quality error reporting with source context and terminal formatting. |
| `dust_plugin_api` | Defines the `DustPlugin` trait and shared workspace analysis contracts. |
| `dust_cli` | The user-facing CLI binary. |

### Dart Environment (`packages/`)

| Package | Responsibility |
| :--- | :--- |
| `dust_dart` | Dart-only annotations and runtime: derive, serde, HTTP, and DB base APIs. |
| `dust_flutter` | Flutter-only annotations and runtime: routing and state management. |
| `dust_db_sqlite3` | sqlite3 driver implementation for Dust DB. |

---

## 🏗️ The 4-Pass Build Pipeline

Every file processed by Dust follows a strict, deterministic sequence to ensure performance and cross-file correctness.

### Pass 1: Discovery & Hashing
The driver resolves the package configuration and calculates a **Build Fingerprint**. This fingerprint includes source text, plugin versions, and tool configuration. If the hash matches the cache, the file is skipped.

### Pass 2: Parse & Shared Workspace Analysis
The tree-sitter backend parses each Dart source file and lowers the syntax into
Dust-owned parser facts. Tree-sitter nodes do not cross the
`dust_parser_dart_ts` crate boundary.
*   **Example:** The Route plugin gathers route facts from parsed/IR data to
    build the unified navigation tree.
*   **Constraint:** Plugins may collect cross-file facts, but they must not
    manually scan raw Dart source or depend on `tree_sitter::Node`.

### Pass 3: Resolution & Lowering
The engine resolves imports, types, annotations, and Dust-owned symbols against
the workspace catalog. The result is the canonical **Dust IR**
(`DartFileIr`), a simplified, language-neutral model that is safe for plugins
to consume. `LibraryIr` exists only as a temporary compatibility alias while the
last external/test callers migrate.

### Pass 4: Validation & Emission
1.  **Validation:** Every plugin runs semantic checks on `DartFileIr` (e.g., "Does this `@Path` param exist in the URL?").
2.  **Generation:** Plugins return generated units/fragments of Dart code from normalized IR and a deterministic symbol plan.
3.  **Assembly:** The `dust_emitter` assembles all fragments into the final `.g.dart` file.

---

## ⚖️ Engineering Standards

### Product Promise

You focus on product. We focus on performance.

App-facing APIs marked stable should not change. When possible, improvements
belong in generated code, runtime internals, or the Rust engine rather than in
migration work for handwritten product code. Features marked 50% stable can
still refine app-facing APIs before stabilization.

> [!IMPORTANT]
> **Performance is a Requirement:**
> All core logic must be validated against the `benchmark_project` (5,000+ files). We target sub-second "warm" rebuild times for any project size.

### 🚫 No-Panic Policy
Never use `.unwrap()` or `.expect()` in plugin code or lowering logic. If an edge case is encountered, emit a `Diagnostic::error` or `Diagnostic::warning`. This ensures a single malformed file doesn't crash the entire build process.

### 🔄 Determinism
The output of `dust build` must be byte-for-byte identical across different machines and runs. Always use `BTreeMap` or sorted collections when iterating over fields or symbols to maintain stable output order.

### ⚡ Fail-Fast Semantics
`--fail-fast` keeps parallel workers enabled. It stops after the first observed worker error, not necessarily the lexically first source file. Requiring strict lexical fail-fast ordering would force serial processing and keep large invalidated builds slower.

### Dust Dart Runtime Gates
Changes to `packages/dust_dart`, `crates/dust_dart_syntax`, or
`crates/dust_dart_emit` must keep the Dart runtime and shared Dart helper crates
fully covered and documented.

Rust helper crates:

```bash
rtk cargo llvm-cov clean --workspace
rtk cargo llvm-cov --no-report -p dust_dart_syntax -p dust_dart_emit
rtk cargo llvm-cov report --summary-only \
  --ignore-filename-regex 'crates/(dust_ir|dust_diagnostics|dust_text)' \
  --fail-under-lines 100 \
  --fail-under-functions 100 \
  --fail-under-regions 100
```

Dart runtime package:

```bash
dart format --set-exit-if-changed packages/dust_dart/lib packages/dust_dart/test
dart analyze packages/dust_dart
dart --enable-asserts test --coverage=packages/dust_dart/coverage packages/dust_dart/test
dart run coverage:format_coverage --check-ignore \
  --packages=.dart_tool/package_config.json \
  --report-on=packages/dust_dart/lib \
  --in=packages/dust_dart/coverage \
  --out=packages/dust_dart/coverage/lcov.info \
  --lcov
awk 'BEGIN{lf=lh=0} /^LF:/{v=$0; sub("LF:","",v); lf+=v} /^LH:/{v=$0; sub("LH:","",v); lh+=v} END{printf("TOTAL LH=%d LF=%d %.2f%%\n", lh, lf, (lf?100*lh/lf:100)); exit(lh==lf && lf>0 ? 0 : 1)}' packages/dust_dart/coverage/lcov.info
```

`packages/dust_dart/analysis_options.yaml` enables
`public_member_api_docs`, so every public runtime API must have Dartdoc before
the analyzer gate passes. Remove generated `.dart_tool`, `build`, `coverage`,
and ignored lockfile artifacts before final status checks.

---

## 🛠️ Contribution Scenarios

| If you want to... | Edit these crates |
| :--- | :--- |
| Support new Dart syntax | `dust_parser_dart_ts`, `dust_parser_dart` |
| Add a new annotation | `packages/`, `dust_ir`, `dust_resolver`, and a plugin crate. |
| Change how code is formatted | `dust_emitter` (Writer/Format modules). |
| Improve error messages | `dust_diagnostics` or the specific plugin's `validate.rs`. |
| Speed up the build | `dust_driver` (caching/worker logic) or `dust_cache`. |

---

## 🚀 Scale Testing

Before submitting changes, run the benchmark test suite to verify there are no performance regressions:

```bash
# 1. Generate 5,000 models
cd examples/benchmark_project
./generate.sh --count 5000

# 2. Benchmark Cold Build
cd ../..
cargo run -p dust_cli -- build --root examples/benchmark_project

# 3. Verify Cache Speed
cargo run -p dust_cli -- build --root examples/benchmark_project

# 4. Verify Invalidated Rebuild Speed
cargo test -p dust_cli benchmark_project_release_build_benchmark -- --ignored --nocapture
```

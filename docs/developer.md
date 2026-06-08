# Developer Guide: Dust Architecture

This document provides a deep dive into the Dust workspace layout, internal build pipeline, and core engineering principles. It is the primary resource for contributors working on the Rust engine or core plugins.

---

## 📂 Workspace Layout

### Rust Engine (`crates/`)

| Crate | Responsibility |
| :--- | :--- |
| `dust_driver` | **The Orchestrator.** Manages the build lifecycle, caching, and worker threads. |
| `dust_parser_dart_ts` | Tree-sitter backend for high-fidelity Dart parsing. |
| `dust_resolver` | Resolves types, imports, and symbols across the workspace. |
| `dust_ir` | The Intermediate Representation used as the contract between the engine and plugins. |
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

### Pass 2: Shared Workspace Analysis
Plugins scan the raw Tree-sitter AST to build a global snapshot of the project.
*   **Example:** The Route plugin finds all `@Route` annotations across all files to build the unified navigation tree.
*   **Constraint:** Plugins must *only* read data in this phase; no IR is generated yet.

### Pass 3: Resolution & Lowering
The engine resolves all types and symbols against the workspace catalog. The resolved AST is then "lowered" into the **Dust IR** (`LibraryIr`), a simplified, language-neutral model that is safe for plugins to consume.

### Pass 4: Validation & Emission
1.  **Validation:** Every plugin runs semantic checks on the IR (e.g., "Does this `@Path` param exist in the URL?").
2.  **Emission:** Plugins return fragments of Dart code (class members, top-level helpers).
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

---

## 🛠️ Contribution Scenarios

| If you want to... | Edit these crates |
| :--- | :--- |
| Support new Dart syntax | `dust_parser_dart_ts`, `dust_parser_dart` |
| Add a new annotation | `packages/`, `dust_resolver`, `dust_ir`, and a new plugin crate. |
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

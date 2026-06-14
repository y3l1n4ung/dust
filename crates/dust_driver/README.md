# 🏎️ dust_driver

The core build engine and orchestration layer for Dust. This crate manages the lifecycle of a generation run, from discovering package roots to writing final files and managing the persistent cache.

## 🏗️ Architectural Role

`dust_driver` is the **conductor** of the engine. It implements the high-level pipeline:
1. **Pass 1: Scan**: Discover all relevant `.dart` files in the workspace.
2. **Pass 2: Parse and Analyze**: Multi-threaded tree-sitter parsing into Dust-owned facts plus global fact collection.
3. **Pass 3: Resolve and Lower**: Enrich canonical `DartFileIr` with symbols, types, imports, and normalized feature config.
4. **Pass 4: Generate and Write**: Plugin-based generation, deterministic assembly, atomic filesystem updates, and cache synchronization.

## 🔑 Key Modules

### `build`
The heart of the generation process.
- **Batch Processing**: Parallelizes the build across all available CPU cores.
- **Incremental Caching**: Uses content hashing and metadata tracking to skip work for unchanged files.
- **Dependency Tracking**: Special logic for Routing and State management to re-trigger builds if global facts change.

### `lower`
The transitional bridge from resolver output to canonical `DartFileIr`. It handles:
- **Type Inference**: Deducing field types for constructor parameters.
- **Inheritance Merging**: Collecting fields from base classes into generated subclasses.
- **Config Normalization**: Moving Dust feature config into structured Rust types.

Issue [#43](https://github.com/y3l1n4ung/dust/issues/43) tracks the migration
that removes the long-lived second model and makes parser/resolver output feed
`DartFileIr` directly.

### `clean`, `watch`, `check`
Secondary engine modes:
- `clean`: Removes all generated artifacts and resets the internal cache.
- `watch`: High-performance filesystem watcher that triggers incremental sub-builds on file change.
- `check`: Validates that generated files on disk match what the engine would produce (ideal for CI).

## 🛡️ Performance Strategy

- **Zero-Copy Intent**: Minimizing redundant string allocations through the pipeline.
- **Work Stealing**: Efficiently distributing files across threads during the scan and emit phases.
- **Cache Locality**: Storing results in `.dart_tool/dust` to minimize disk search time.

---
*The `dust_driver` is consumed by the `dust_cli` crate, which provides the user-facing command line interface.*

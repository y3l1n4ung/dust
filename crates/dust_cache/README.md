# 💾 dust_cache

The persistent caching layer for the Dust engine. This crate manages the `.dart_tool/dust` directory, storing fingerprints and pre-parsed snapshots to enable high-speed incremental builds.

## 🏗️ Architectural Role

`dust_cache` is the **memory** of the engine. It allows Pass 1 (Discovery) and Pass 2 (Analyze) to skip work for files that haven't changed. By storing expensive-to-compute data like content hashes and analysis snapshots, it reduces subsequent build times from seconds to milliseconds.

## 🔑 Key Concepts

### `WorkspaceCache`
The primary handle for cache operations.
- **Atomic Access**: Ensures that concurrent build processes don't corrupt the cache.
- **Key-Value Storage**: Maps source file paths to `CacheEntry` structures.

### `CacheEntry`
A serialized snapshot of a file's state at the time of the last build. It includes:
- **Source Hash**: BLAKE3 hash of the original `.dart` content.
- **Output Hash**: Hash of the previously generated `.g.dart` content.
- **Analysis Snapshot**: The plugin facts (Pass 2 output) extracted from the file.
- **Config Hash**: A hash of the project's configuration (e.g., `pubspec.yaml`), ensuring the cache is invalidated if global settings change.

## 🛡️ Cache Invalidation Strategy

A cache entry is considered valid only if:
1. The current source file content matches the stored `source_hash`.
2. The project's configuration matches the stored `config_hash`.
3. The Dust tool binary version/hashes match (Pass 1 tool integrity).
4. No dependent global facts have changed (handled by `dust_driver`).

## 🚀 Performance Impact

On a project with 5,000 files:
- **Cold Build**: ~1.3 seconds (full parsing and analysis).
- **Warm Build (No Changes)**: **~50ms** (metadata check only).
- **Incremental Build (1 File Change)**: ~150ms (re-parse only the changed file and its dependents).

---
*The `dust_cache` uses the `bincode` and `serde` crates for high-speed binary serialization.*

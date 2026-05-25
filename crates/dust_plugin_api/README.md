# 🧩 dust_plugin_api

The foundational contract and integration layer for Dust plugins. This crate defines the interfaces that allow independent feature generators (like SerDe, Routing, or Data Classes) to communicate with the core build driver and the final emitter.

## 🏗️ Architectural Role

`dust_plugin_api` acts as the **mediator** in the 4-pass pipeline. It provides:
1. **Abstraction**: Decouples specific generator logic from the build orchestration.
2. **Global Context**: Mechanisms for plugins to share "facts" across files (e.g., "Class X is copyable").
3. **Symbol Management**: A reservation system to prevent name collisions in generated code.

## 🔑 Key Components

### `DustPlugin` Trait
The primary interface for all features. It defines three distinct phases:
- `validate`: Static analysis of the IR before any code is generated.
- `collect_workspace_analysis`: (Pass 2) Global fact gathering across the entire project.
- `emit`: (Pass 3) Rendering IR into Dart code fragments based on a finalized `SymbolPlan`.

### `WorkspaceAnalysis`
A multi-threaded, append-only collection system.
- **Builder**: Used during scanning to record facts keyed by strings.
- **Snapshot**: A serialized, per-file subset used for incremental build caching.
- **Immutable Analysis**: The final set of global facts provided to plugins during emission.

### `PluginContribution`
The unit of generated output. Instead of writing full files, plugins return "contributions" (mixins, top-level functions, support types) which the `dust_emitter` merges into the final `.g.dart` file.

### `SymbolPlan`
A deterministic registry of reserved names (e.g., `_$User`, `_undefined`). It ensures that multiple plugins can safely generate code into the same scope without clobbering each other.

## 🛠️ Implementation Rules

- **Deterministic**: Plugins must be pure functions of `(IR, SymbolPlan)`. No side effects or non-deterministic string generation.
- **Lazy**: Heavy computation should happen during `emit`, not `validate`.
- **Surgical**: Plugins should only generate code for classes/enums they "own" (i.e., those carrying their specific annotations).

---
*For a step-by-step guide on creating new plugins, see [../../docs/plugin-guide.md](../../docs/plugin-guide.md).*

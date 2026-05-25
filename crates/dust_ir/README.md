# 🌲 dust_ir

The Semantic Intermediate Representation for Dust. This crate defines the "source of truth" for the engine—a normalized, type-safe view of Dart source code after resolution but before emission.

## 🏗️ Architectural Role

`dust_ir` sits at the heart of the engine. It transforms raw syntax (provided by the parser) and resolved symbols (provided by the resolver) into a stable, easy-to-consume tree for plugins. It eliminates the complexities of the Dart AST, exposing only the structures relevant to code generation.

## 🔑 Key Concepts

### `LibraryIr`
The root container for a single `.dart` file. It carries:
- Normalized classes and enums.
- Package metadata (name, root).
- Resolved imports.
- Source spans for high-quality diagnostics.

### `TypeIr`
A powerful, recursive representation of Dart types. It handles:
- **Built-ins**: `int`, `String`, `bool`, etc.
- **Named Types**: User-defined classes with generic arguments.
- **Nullability**: Deep tracking of nullable vs. non-nullable types.
- **Functional/Records**: Advanced Dart syntax normalized into simple shapes.

### `Serde` & `Derive` Metadata
Normalization of plugin-specific configurations. Instead of plugins parsing raw annotation strings, the `dust_driver` lowers those arguments into specialized IR structures (e.g., `SerdeFieldConfigIr`).

## 🛡️ Design Goals

1. **Stability**: Changes to Dart syntax or the parser backend should not break plugin logic if the semantic meaning remains the same.
2. **Efficiency**: Designed for fast cloning and deep equality checks, which power the incremental build cache.
3. **Completeness**: Carries enough information (via `SpanIr`) to map errors in generated code back to the original user source.

## 🛠️ Usage

This crate is **read-only** for plugins. Plugins receive a reference to `LibraryIr` and should treat it as immutable. Modifications to the IR should only occur in the `dust_driver::lower` module.

---
*For details on how syntax is converted to IR, see the `dust_driver` documentation.*

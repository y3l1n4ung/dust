# 🌲 dust_ir

The Semantic Intermediate Representation for Dust. This crate defines the "source of truth" for the engine—a normalized, type-safe view of Dart source code after resolution but before emission.

## 🏗️ Architectural Role

`dust_ir` sits at the heart of the engine. It transforms raw syntax (provided by the parser) and resolved symbols (provided by the resolver) into a stable, easy-to-consume tree for plugins. It eliminates the complexities of the Dart AST, exposing only the structures relevant to code generation.

## 🔑 Key Concepts

### `DartFileIr`
The canonical root container for a single `.dart` file. It carries:
- Normalized classes and enums.
- Package metadata (name, root).
- Resolved imports.
- Parser-owned directives, annotations, query calls, and declaration containers.
- Source spans for high-quality diagnostics.

`LibraryIr` is a temporary compatibility alias for older tests and external
callers; production crates should use `DartFileIr`.

### `TypeIr`
A powerful, recursive representation of Dart types. It handles:
- **Built-ins**: `int`, `String`, `bool`, etc.
- **Named Types**: User-defined classes with generic arguments.
- **Nullability**: Deep tracking of nullable vs. non-nullable types.
- **Functional/Records**: Advanced Dart syntax normalized into simple shapes.

### Annotation, Type, and Feature Metadata
Parser-owned annotation/type facts are normalized into IR before plugins run.
Instead of plugins parsing raw annotation strings, resolver/lowering fills
specialized IR structures such as `SerdeFieldConfigIr`.

## 🛡️ Design Goals

1. **Stability**: Changes to Dart syntax or the parser backend should not break plugin logic if the semantic meaning remains the same.
2. **Efficiency**: Designed for fast cloning and deep equality checks, which power the incremental build cache.
3. **Completeness**: Carries enough information (via `SpanIr`) to map errors in generated code back to the original user source.

## 🛠️ Usage

This crate is **read-only** for plugins. Plugins receive a reference to
`DartFileIr` and should treat it as immutable. During the refactor, resolver and
driver lowering are responsible for enriching the model before plugin
validation/generation.

---
*For details on how syntax is converted to IR, see the `dust_driver` documentation.*

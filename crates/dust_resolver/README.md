# 🔍 dust_resolver

The semantic mapping layer of the Dust engine. This crate is responsible for taking the raw output of the parser and resolving it into semi-semantic structures, identifying which annotations correspond to known Dust symbols.

## 🏗️ Architectural Role

`dust_resolver` acts as **Pass 2.5** in the pipeline. It sits between the low-level parser (which only understands syntax) and the high-level IR (which understands intent). Its primary job is to answer the question: *"Does this `@Something` annotation actually mean anything to Dust?"*

## 🔑 Key Components

### `SymbolCatalog`
The central registry of "known" symbols. It maps short, surface-level annotation names (e.g., `Serialize`) to their fully qualified, unique internal IDs (e.g., `derive_serde_annotation::Serialize`).
- Supports **Traits**: Symbols that change class behavior (e.g., `Eq`).
- Supports **Configs**: Symbols that provide settings (e.g., `SerDe`).

### `ResolvedLibrary`
A specialized view of a library where:
- All directives (imports/parts) have been identified.
- Annotations have been resolved to `SymbolId`s.
- Class/Field/Method structures have been flattened for easier lowering.

### `resolve_library`
The entry point function. It performs a "pre-lowering" pass that:
1. Filters out non-generation relevant syntax.
2. Validates that `part` directives match the expected `.g.dart` naming convention.
3. Produces a `ResolveResult` containing both the resolved structures and any resolution-time diagnostics (e.g., "Unknown symbol").

## 🛡️ Design Principles

- **Fail-Safe**: Resolution should never panic. If a symbol is unknown, it simply emits a warning and continues. This allows Dust to coexist with other code generators (like `json_serializable`).
- **Path Sensitive**: Handles workspace-relative path resolution to ensure imports and parts are correctly mapped across different package structures.

---
*Next Step: The output of this crate is consumed by `dust_driver::lower` to produce the final `LibraryIr`.*

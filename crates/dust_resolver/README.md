# 🔍 dust_resolver

The semantic mapping layer of the Dust engine. This crate enriches parser-owned
Dart file facts with imports, symbols, annotation identities, and normalized
feature configuration.

## 🏗️ Architectural Role

`dust_resolver` acts as **Pass 2.5** in the pipeline. It sits between the
low-level parser, which only understands syntax, and plugin validation/generation,
which consume canonical `DartFileIr`. Its primary job is to answer the question:
*"Does this `@Something` annotation actually mean anything to Dust?"*

## 🔑 Key Components

### `SymbolCatalog`
The central registry of "known" symbols. It maps short, surface-level annotation names (e.g., `Serialize`) to their fully qualified, unique internal IDs (e.g., `dust_dart::Serialize`).
- Supports **Traits**: Symbols that change class behavior (e.g., `Eq`).
- Supports **Configs**: Symbols that provide settings (e.g., `SerDe`).

### Resolved `DartFileIr`
A specialized view of a Dart file where:
- All directives (imports/parts) have been identified.
- Annotations have been resolved to `SymbolId`s.
- Class/Field/Method structures have been flattened for easier lowering.

### `resolve_library`
The entry point function. It performs a "pre-lowering" pass that:
1. Filters out non-generation relevant syntax.
2. Validates that `part` directives match the expected `.g.dart` naming convention.
3. Produces a `ResolveResult` containing enriched file structures and any resolution-time diagnostics (e.g., "Unknown symbol").

## 🛡️ Design Principles

- **Fail-Safe**: Resolution should never panic. If a symbol is unknown, it simply emits a warning and continues. This allows Dust to coexist with other code generators (like `json_serializable`).
- **Path Sensitive**: Handles workspace-relative path resolution to ensure imports and parts are correctly mapped across different package structures.

---
*Next Step: This crate is being migrated under Issue #43 so resolver enrichment
returns canonical `DartFileIr` directly instead of a second long-lived model.*

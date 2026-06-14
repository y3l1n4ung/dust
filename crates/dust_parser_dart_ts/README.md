# 🛠️ dust_parser_dart_ts

The high-performance syntax extraction backend for Dust, powered by **Tree-sitter**. This crate is responsible for the actual parsing of `.dart` source files and extracting the data structures needed for code generation.

## 🏗️ Architectural Role

This is the primary implementation of the `dust_parser_dart` backend interface.
It performs the "heavy lifting" of Pass 2, scanning user code and translating
tree-sitter nodes into Dust-owned parser facts. `tree_sitter::Node` is private
to this crate; callers receive parser surfaces and, as the refactor progresses,
canonical `DartFileIr` data.

## 🔑 Key Components

### `TreeSitterDartBackend`
The main entry point that wraps the `tree-sitter-dart` grammar. It provides:
- Multi-threaded safe parsing.
- Error recovery (gracefully handles partially invalid Dart code).
- Incremental parsing support (via Tree-sitter's internal mechanisms).

### `extract_classes`, `extract_enums`, `extract_directives`, `extract_query_calls`
Specialized modules that navigate the AST to find generation-relevant declarations. They use efficient tree-walking patterns to collect:
- Metadata annotations (Pass 2 input).
- Class and field names.
- Type shapes and compatibility `type_source` values from tree-sitter type nodes.
- Preserved expression source spans for syntax Dust intentionally does not interpret.
- Constructor signatures.
- Dust DB query helper calls.

## 🛡️ Design Principles

1. **Extraction-Only**: This crate does **not** perform semantic analysis or type checking. It only reports what is syntactically present in the file.
2. **Resilience**: It is designed to be highly tolerant of syntax errors. If a file is unparseable, it returns a diagnostic warning rather than crashing.
3. **Speed**: Leverages Tree-sitter's incremental parsing and Rust's safety to ensure that Pass 2 remains a sub-100ms phase for even large project sub-builds.
4. **Boundary Discipline**: Tree-sitter nodes and cursors never leave this crate.

## 🛠️ Usage

This crate should generally only be used by the `dust_driver`. Other crates should depend on `dust_parser_dart` (for interfaces) or `dust_ir` (for semantic data).

---
*Note: This backend uses the `tree-sitter-dart` C grammar via Rust FFI.*

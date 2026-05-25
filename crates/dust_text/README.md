# 📖 dust_text

The foundational text processing and indexing layer for Dust. This crate provides the primitive types for managing source code as memory-efficient, indexed text.

## 🏗️ Architectural Role

`dust_text` is the **base layer** of the engine. It provides the vocabulary for talking about "where" things are in a Dart file. By centralizing text math (offsets, ranges, line/column mapping), it ensures that diagnostics and parser results are consistent across the entire pipeline.

## 🔑 Key Components

### `SourceText`
The primary container for a single file's content.
- **Line Indexing**: Efficiently maps byte offsets to line/column pairs for human-readable error reporting.
- **Slicing**: Provides safe, range-checked views of the source text.
- **Arc-backed**: Designed to be shared across threads without copying the underlying string.

### `TextRange` & `TextSize`
Type-safe byte offsets and ranges.
- Prevents "off-by-one" errors common with raw integer offsets.
- Provides robust addition and subtraction for range manipulation.

### `FileId`
A unique, integer-based identifier for every file in the build session. This allows the engine to carry location data (spans) throughout the IR without needing to store full file paths everywhere.

## 🛡️ Performance Strategy

- **Lazy Indexing**: Line indices are only computed if a diagnostic is actually emitted for that file.
- **Compact Spans**: `TextRange` and `FileId` are kept as small as possible to minimize the memory footprint of the IR tree.
- **UTF-8 Awareness**: All range math is byte-oriented but ensures boundaries land on valid UTF-8 character starts.

---
*Next Step: This crate is used by `dust_parser_dart` and `dust_diagnostics` to build semantic spans.*

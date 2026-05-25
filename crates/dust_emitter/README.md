# 📝 dust_emitter

The final assembly and rendering stage of the Dust engine. This crate takes multiple generated fragments from plugins and merges them into high-quality, formatted Dart source files.

## 🏗️ Architectural Role

`dust_emitter` is the **Pass 4** in the pipeline. It is responsible for the physical layout and formatting of the `.g.dart` files. It ensures that the final output is not only functionally correct but also readable and compliant with Dart's style guidelines.

## 🔑 Key Concepts

### `DartWriter`
A high-level abstraction for writing Dart code. It handles:
- **Indentation Tracking**: Automatic management of curly brace depths.
- **Block Layout**: Helper methods for classes, mixins, and functions.
- **Section Merging**: Intelligent placement of imports, parts, and declarations.

### `MergedSections`
A container that reconciles contributions from multiple plugins. Since multiple plugins might want to add members to the same class (e.g., both `ToString` and `Serialize` adding code to class `User`), this module groups them by target and ensures they are emitted in a deterministic order.

### `GENERATED_HEADER`
A unified header string (shared via `dust_plugin_api`) that is prepended to every file. It includes standard linter suppressions (e.g., `// ignore_for_file: type=lint`) to ensure a warning-free developer experience.

## ⚙️ Emission Logic

1. **Reconciliation**: Group plugin contributions by class name.
2. **Buffering**: Render sections into an in-memory buffer using `DartWriter`.
3. **Normalization**: Apply internal Rust-based normalization (trimming, newline stabilization) to ensure deterministic output without the overhead of an external formatter.
4. **Change Detection**: Compare the new buffer against existing file contents to avoid unnecessary disk writes (Pass 4).

## 🛡️ Performance Strategy

- **Pre-sized Buffers**: Uses pre-allocated `String` buffers to minimize reallocations during assembly.

---
*The `dust_emitter` output is handed back to the `dust_driver` for the final filesystem write.*

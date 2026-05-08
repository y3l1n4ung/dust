# Implementation Plan: Global Symbol Table & String Interning

## Background & Motivation
Despite heavy parallelization, the engine's full-build time for 5,000 files is ~14s, falling short of the < 1.2s target. The remaining bottlenecks are CPU and memory overhead caused by allocating thousands of `String` and `Box<str>` objects per file in the AST/IR, as well as a lack of a global workspace dependency graph for fast O(1) inheritance resolution.

## Scope & Impact
This redesign shifts the `dust` compiler frontend to a 2-pass architecture with zero-cost string interning. 
- **String Interning:** All identifiers in the IR and AST will be replaced with lightweight `u32` handles (`lasso::Spur`).
- **Global Resolution:** A shared `GlobalSymbolTable` will track all declarations across the workspace.

## Proposed Solution & Phased Implementation Plan

### Phase 1: Core Dependencies & Types
1. **Dependencies:** Add `lasso = "0.7"` to `[workspace.dependencies]` in `Cargo.toml`.
2. **Interner:** Introduce a `dust_resolver::Symbol` newtype (wrapping `lasso::Spur`) and a thread-safe interner (`lasso::ThreadedRodeo`).
3. **IR Refactoring:** Update `crates/dust_ir/src/` to replace `String` and `Box<str>` fields in `ClassIr`, `FieldIr`, `TypeIr`, and `SymbolId` with the new `Symbol` type.

### Phase 2: Global Symbol Table
1. **Symbol Tracking:** Expand `crates/dust_resolver/src/catalog.rs` to include a `GlobalSymbolTable`.
2. **Data Structure:** The table will map an interned `Symbol` (representing a class or enum name) to its semantic metadata (e.g., its superclass `Symbol`, traits, and origin library). This enables O(1) cross-library inheritance resolution without scanning source strings.

### Phase 3: 2-Pass Driver Orchestration
1. **Refactor Driver:** Update `crates/dust_driver/src/build/batch.rs` to split work into two parallel passes.
2. **Pass 1 (Scan):** Parse all changed and uncached files. Extract top-level declarations, intern their names into the shared `ThreadedRodeo`, and populate the `GlobalSymbolTable`.
3. **Pass 2 (Lowering):** With the global table locked for reading, run the lowering phase in parallel. The engine can now securely and instantly resolve superclass fields across library boundaries.

### Phase 4: Emitter Adaptation
1. **String Resolution:** Update `crates/dust_plugin_api` and `crates/dust_emitter` to accept a reference to the `lasso::ThreadedRodeo`.
2. **Code Generation:** When writing `.g.dart` files, the emitter will resolve the `Symbol` handles back into `&str` slices, completely avoiding intermediate string allocations during generation.

## Verification
- Run `cargo check` to ensure the type migration from `String` to `Symbol` is complete across all 14 crates.
- Run `cargo test` to verify that `dust_plugin_serde` and `dust_plugin_derive` still emit correct code.
- Run the 5,000-file benchmark to confirm the full build completes in < 1.2s.
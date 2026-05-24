# Plugin Guide: Extending Dust

This guide covers the concrete steps for adding a new annotation plugin to the Dust engine without breaking the shared pipeline rules.

---

## Use A New Plugin Only When...

*   The feature has its own distinct **Dart annotation surface**.
*   The generation logic is complex enough to require its own **ownership boundary**.
*   The feature needs **shared workspace analysis** (Pass 2) to gather facts across files.

> [!TIP]
> If you're adding a new behavior to an existing annotation (like adding a new property to `@SerDe`), extend the existing `dust_plugin_serde` crate instead of creating a new one.

---

## 🏗️ Recommended Crate Structure

To maintain consistency across the engine, all plugins should follow this module split:

| Module | Responsibility |
| :--- | :--- |
| `plugin.rs` | Implementation of the `DustPlugin` trait and registration entrypoint. |
| `validate.rs` | Semantic validation (e.g., "Are these annotation arguments valid?"). |
| `emit.rs` | Logic for generating Dart code fragments and class members. |
| `analysis.rs` | Logic for Pass 2 workspace scanning (collecting cross-file facts). |
| `tests/` | Focused integration tests verifying IR-to-Dart emission. |

---

## 🚀 Step-by-Step Implementation

### 1. Add the Public Dart API
Create a new package under `packages/` (or extend an existing one).
*   Define the annotation class (e.g., `@MyFeature`).
*   Ensure it uses the standard `DeriveTrait` or `DeriveConfig` base classes.
*   Add Dartdoc and run `dart analyze` to ensure a clean public surface.

### 2. Implement the `DustPlugin` Trait
In your Rust crate, implement the core plugin contract:

```rust
impl DustPlugin for MyPlugin {
    fn plugin_name(&self) -> &'static str { "dust_plugin_my_feature" }

    // Claim your annotations so no other plugin can use them
    fn claimed_traits(&self) -> Vec<SymbolId> {
        vec![SymbolId::new("my_package::MyTrait")]
    }

    // PASS 2: Collect facts from raw source (optional)
    fn collect_workspace_analysis(&self, library: &ParsedLibrarySurface, analysis: &mut WorkspaceAnalysisBuilder) { ... }

    // PASS 4: Validate the lowered IR
    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> { ... }

    // PASS 4: Generate code fragments
    fn emit(&self, library: &LibraryIr, plan: &SymbolPlan) -> PluginContribution { ... }
}
```

### 3. Register the Plugin
Wire your new crate into the `dust_driver` orchestrator.
*   **File:** `crates/dust_driver/src/build/support.rs`
*   **Action:** Add your plugin to the `default_registry()` function.

> [!IMPORTANT]
> **Don't Forget the Fingerprint:**
> Update `CODEGEN_FINGERPRINT_INPUT` in the driver to include your plugin's source files. This ensures users get a fresh rebuild when you update the plugin's Rust code.

---

## ⚖️ Best Practices for Plugin Authors

### 🛡️ Isolation & Namespacing
Because the `dust_emitter` merges all plugin contributions into a single file scope, you **must** namespace your private generated helpers.
*   **Bad:** `_parseJson(json)`
*   **Good:** `_$MyPlugin_parseJson(json)`

### 🚫 Panic Safety
Plugins run in parallel worker threads. A single `panic!()` will crash the entire build process.
*   **Always** return a `Vec<Diagnostic>` for errors found during validation.
*   **Never** use `.unwrap()` or `.expect()` on data derived from user source code.

### 🔄 Use Shared Analysis
If your plugin needs to know about other files (e.g., "Find all classes marked with X"), use `collect_workspace_analysis`.
*   **Do not** perform custom file I/O or manual scanning inside `emit()`.
*   Read the results from the `SymbolPlan` provided during the emit phase.

---

## ✅ Pre-Commit Checklist

- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy` is clean (no warnings).
- [ ] New feature is used in `examples/product_showcase` and verified with `dart analyze`.
- [ ] Documentation updated in `docs/usage/` if the public API changed.

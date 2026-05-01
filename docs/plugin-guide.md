# Plugin Guide

This guide covers the concrete steps for adding a new Dust annotation plugin
without breaking the shared pipeline rules.

## Use A New Plugin Only When

- the feature has its own Dart annotation surface or config contract
- generation logic does not fit cleanly inside `dust_plugin_derive` or
  `dust_plugin_serde`
- the emitted helpers, validation, or shared analysis are distinct enough to
  justify a new ownership boundary

If the change is only new behavior for an existing derive or serde annotation,
extend the existing plugin crate instead.

## Files You Usually Touch

1. Dart annotation package in `packages/`
2. Rust plugin crate in `crates/`
3. driver registration in
   `crates/dust_driver/src/build/support.rs`
4. example coverage in `examples/product_showcase/`
5. stress coverage in `examples/stress_project/` when the feature changes
   cache shape, throughput, or shared analysis

## Recommended Crate Shape

Follow the same split used by the built-in plugins.

- `plugin.rs`: `DustPlugin` implementation and registration entrypoint
- `validate.rs`: annotation-level validation
- `emit.rs` plus focused helpers: generated member assembly
- `analysis.rs`: shared workspace analysis keys and parse-only collection logic
- `README.md`: ownership, responsibilities, and edit hints
- `tests/`: focused integration tests grouped by feature area

Keep large emitters or validation matrices broken into feature files instead of
letting one module grow unchecked.

## Step-By-Step

### 1. Add the public Dart API

Create or extend the right package under `packages/` with:

- the annotation class or config type
- Dartdoc for every public API
- `dart analyze` and `dart test` coverage

Keep the Dart surface minimal. Resolver and plugin logic should depend on
stable symbols, not text matching.

### 2. Reserve symbol ownership

In the Rust plugin crate, implement `DustPlugin` from `dust_plugin_api`.

Use:

- `claimed_traits()` for derive-style marker annotations
- `claimed_configs()` for configuration annotations
- `requested_symbols()` for helper names that must be reserved before emit

`PluginRegistry` enforces exclusive ownership, so every new trait or config
symbol must have one clear plugin owner.

### 3. Keep cross-file work in shared analysis

If the feature needs workspace-wide facts:

- collect them from `ParsedLibrarySurface`
- write them into `WorkspaceAnalysisBuilder`
- read them later from `SymbolPlan::workspace_analysis()`

Do not add plugin-specific scan branches to `dust_driver`. The driver already
calls `collect_workspace_analysis()` for every registered plugin during the
shared scan phase.

### 4. Register the plugin in the default driver pipeline

Wire the plugin into `crates/dust_driver/src/build/support.rs`.

- add its source files to `CODEGEN_FINGERPRINT_INPUT`
- register it in `default_registry()`

If you skip the fingerprint update, cache invalidation will miss plugin source
changes.

### 5. Add real example coverage

Use `examples/product_showcase/` for correctness:

- analyzer-clean generated output
- runtime tests for the public behavior
- one or more representative models using the new annotation

Use `examples/stress_project/` when the feature affects:

- shared workspace analysis
- cache metadata or cache reuse
- build throughput
- linked generated models across files

### 6. Add focused Rust tests

Cover the plugin in layers:

- plugin API or registry tests for ownership and ordering behavior
- plugin crate tests for validation and emitted members
- driver tests only when build scheduling, cache behavior, or workspace
  orchestration changes

Prefer several small test files over one monolithic matrix.

## Minimal Registration Skeleton

```rust
use dust_diagnostics::Diagnostic;
use dust_ir::{LibraryIr, SymbolId};
use dust_parser_dart::ParsedLibrarySurface;
use dust_plugin_api::{
    DustPlugin, PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder,
};

pub struct MyPlugin;

impl DustPlugin for MyPlugin {
    fn plugin_name(&self) -> &'static str {
        "dust_plugin_my_feature"
    }

    fn claimed_traits(&self) -> Vec<SymbolId> {
        vec![SymbolId::new("my_annotation::MyTrait")]
    }

    fn collect_workspace_analysis(
        &self,
        library: &ParsedLibrarySurface,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        let _ = (library, analysis);
    }

    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        let _ = library;
        Vec::new()
    }

    fn emit(&self, library: &LibraryIr, plan: &SymbolPlan) -> PluginContribution {
        let _ = (library, plan);
        PluginContribution::default()
    }
}
```

## Pre-Commit Checklist

- `cargo test --workspace --quiet`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- package/example `dart analyze`
- package/example `dart test`
- update `README.md`, crate `README.md`, and roadmap docs when the public
  surface changes

For general repository rules, see [../CONTRIBUTING.md](../CONTRIBUTING.md) and
for pipeline ownership, see [developer.md](developer.md).

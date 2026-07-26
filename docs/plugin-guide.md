# Plugin Guide

Dust plugins are Rust crates compiled into the CLI. They claim Dart annotation
symbols, validate canonical `DartFileIr`, and return generated contributions for
the shared emitter. Dust does not currently load third-party plugins at runtime.

For the complete build pipeline and crate map, see the
[Developer Guide](developer.md).

## When to Add a Plugin

Add a plugin when a feature owns a distinct annotation surface and generation
boundary. Extend an existing plugin when the change belongs to annotations it
already owns.

> [!TIP]
> A runtime-only Dart or Flutter feature does not need a Rust plugin. Add a
> plugin only when Dust must inspect source or generate code.

## Recommended Structure

Use the smallest structure that keeps parsing, validation, and emission separate:

| Path | Responsibility |
| :--- | :--- |
| `src/lib.rs` | Public plugin type and registration export. |
| `src/plugin.rs` | `DustPlugin` implementation and module wiring. |
| `src/plugin/constants.rs` | Claimed symbols, supported annotation names, and analysis keys. |
| `src/plugin/model.rs` | Feature-specific models shared across phases. |
| `src/plugin/parse.rs` | Converts normalized IR into feature models. |
| `src/plugin/validate.rs` | Returns diagnostics for invalid source. |
| `src/plugin/emit.rs` | Produces `PluginContribution` values. |
| `src/plugin/analysis.rs` | Optional cross-file fact collection. |
| `tests/` | Registration, validation, analysis, and exact-output tests. |

Small plugins may combine modules. Do not create empty layers only to match this
table.

## 1. Add the Dart Annotation

Place Dart-only annotations in `packages/dust_dart` and Flutter-dependent
annotations in `packages/dust_flutter`.

Derive-style traits and configuration extend `DeriveTrait` or `DeriveConfig`.
Other features may use a standalone const annotation class, as routing and state
do. Keep the public API small and document the generated contract.

Run `dart analyze` for a Dart package or `flutter analyze` for a Flutter package.

## 2. Implement `DustPlugin`

Every plugin consumes canonical IR. Parser nodes and source-string parsing stay
inside the parser and resolver layers.

```rust
use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::{DustPlugin, PluginContext, PluginContribution};

pub struct MyPlugin;

impl DustPlugin for MyPlugin {
    fn plugin_name(&self) -> &'static str {
        "MyFeature"
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        &["dust_dart::MyFeature"]
    }

    fn supported_annotations(&self) -> &'static [&'static str] {
        &["MyFeature"]
    }

    fn validate(&self, file: &DartFileIr) -> Vec<Diagnostic> {
        validate_my_feature(file)
    }

    fn generate(
        &self,
        file: &DartFileIr,
        context: &PluginContext<'_>,
    ) -> Vec<PluginContribution> {
        vec![emit_my_feature(file, context.symbol_plan)]
    }
}
```

Use `claimed_traits()` for annotations placed inside `@Derive([...])` and
`claimed_configs()` for configuration annotations. Fully qualified names become
the resolver's canonical symbol IDs. Duplicate ownership fails during registry
construction.

`supported_annotations()` contains the short surface names used by workspace
discovery. Claiming a symbol does not add its short name automatically.

Use `partless_configs()` only when an annotation is valid without a source
`part '<name>.g.dart';` declaration.

## Feature Lowering Boundary

Dust does not expose a separate plugin lowering hook today. The supported
extension point is normalized IR:

1. Parser crates extract syntax facts from Dart source.
2. `dust_resolver` resolves symbols and turns annotation meaning into typed IR.
3. `dust_driver` lowers resolved declarations and wires diagnostics, cache, and
   workspace flow.
4. Plugins validate `DartFileIr` and return generated contributions.

If a plugin needs source information that is not present in `DartFileIr`, add a
parser fact, resolver normalization, or IR field first. Do not add feature
branches to driver lowering.

> [!IMPORTANT]
> Driver lowering must stay feature-neutral. SerDe, DB, routing, state, HTTP,
> validation, and derive behavior belongs in the owning resolver facts and
> plugin crate.

Add a new plugin lowering hook only after a concrete feature cannot be expressed
as normalized IR plus plugin validation/generation. The hook should be small,
deterministic, tested in `dust_plugin_api`, and proven by migrating one existing
feature path.

## 3. Return Contributions

`PluginContribution` supports these output sections:

- `mixin_members` for members attached to a source class
- `shared_helpers` and `support_types` for generated declarations
- `top_level_functions` for library-level functions
- `primary_source` when a feature owns the complete primary output
- `auxiliary_outputs` for additional generated files
- `diagnostics` found while preparing output

The emitter merges contributions in registry order. Use shared rendering helpers
from `dust_dart_emit`, choose deterministic names, and keep all collection order
stable. Reserve generated names in `requested_symbols()` when another plugin or
validation phase needs to see them through `SymbolPlan`.

> [!IMPORTANT]
> Return diagnostics for invalid user source. Do not use `unwrap()` or `expect()`
> on values derived from Dart input.

## 4. Collect Cross-File Facts

When generation depends on declarations in other libraries, implement
`collect_workspace_analysis_ir()` and add deterministic values to a versioned
analysis key:

```rust
fn collect_workspace_analysis_ir(
    &self,
    file: &DartFileIr,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    collect_my_feature_facts(file, analysis);
}
```

The driver merges facts from pending libraries with cached per-library snapshots.
Generators read the immutable result through
`context.symbol_plan.workspace_analysis()` or `workspace_string_set()`.

Do not scan workspace files or reopen Dart source inside `generate()`.

## 5. Register the Plugin

Wire the crate through all compiled surfaces:

1. Add the crate to the root Cargo workspace.
2. Add it as a dependency of `dust_driver`.
3. Export a `register_plugin()` function from the plugin crate.
4. Register it in
   `crates/dust_driver/src/build/support/registry.rs` in the intended order.
5. If the plugin uses generation assets outside the registered codegen source
   roots, add that root to `CODEGEN_FINGERPRINT_ROOTS` in
   `crates/dust_driver/build.rs`.

Dust automatically fingerprints Rust source and templates under registered
codegen roots. Adding a module or template under a normal plugin `src/` tree
does not require editing the cache hash code.

## 6. Test the Contract

Cover the boundaries that the plugin owns:

- registration, claimed symbols, and supported annotation names
- valid and invalid normalized IR
- diagnostics with useful source spans
- workspace analysis and cached-fact behavior when applicable
- exact emitted output, including imports and helper names
- driver integration for discovery, cache invalidation, and generated files
- one real example using the public Dart or Flutter API

Generated Dart fixture snapshots use `.dart.snapshot`, not `.dart` or
`.g.dart`. Prefer exact snapshot equality over partial `contains` assertions for
generated output.

## Validation

Run the narrow plugin checks first, then the repository gates appropriate to the
change:

```bash
cargo fmt --all -- --check
cargo test -p dust_plugin_my_feature
cargo clippy -p dust_plugin_my_feature --all-targets -- -D warnings
```

Also analyze and test the affected Dart package or Flutter example. Public
behavior changes require an update to the relevant [usage guide](usage/README.md)
and example.

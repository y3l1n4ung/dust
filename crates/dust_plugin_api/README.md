# dust_plugin_api

Shared contract between Dust plugins, the driver, and the emitter.

Plugins implement `DustPlugin` to collect workspace facts, validate IR, and
return generated contributions.

## Owns

- `DustPlugin`
- plugin validation and generation result types
- workspace analysis containers
- generated contribution structures
- symbol reservation helpers
- shared generated-file header constants

## Plugin Flow

1. `collect_workspace_analysis` records cross-file facts.
2. `validate` reports feature-specific diagnostics before output is written.
3. `generate` returns ordered `PluginContribution` values for the emitter.

`generate` is the only output hook. Plugins should generate from `DartFileIr`
and `PluginContext`, not by reparsing raw Dart source.

## Design Rules

- Keep plugin output deterministic.
- Put shared cross-file facts in workspace analysis.
- Reserve generated names through the shared symbol plan.
- Keep feature-specific logic inside the owning plugin crate.

See [`../../docs/plugin-guide.md`](../../docs/plugin-guide.md) for adding a new
plugin.

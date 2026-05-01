# dust_plugin_api

Shared plugin contracts for Dust.

## Owns

- plugin registration
- symbol plans
- shared workspace analysis builder and immutable analysis
- plugin contribution interfaces

## Used by

- `dust_driver`
- `dust_plugin_derive`
- `dust_plugin_serde`
- `dust_emitter`

## Edit here when

- new plugin hooks are needed
- cross-file shared analysis rules change
- plugins need new plan-time data from the driver

See [../../docs/plugin-guide.md](../../docs/plugin-guide.md) for the full
plugin authoring workflow and driver registration checklist.

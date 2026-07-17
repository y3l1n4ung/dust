# dust_cache

Persistent cache support for the Dust engine.

This crate owns the `.dart_tool/dust` cache data used by `dust_driver` to skip
unchanged analysis and generation work.

## Owns

- source fingerprints
- generated output fingerprints
- per-library analysis snapshots
- config and tool identity data used for invalidation
- atomic cache reads and writes

## Cache Validity

A cache entry is reusable only when the current inputs match the persisted
entry:

- source content hash
- relevant package/config hash
- Dust tool identity
- generated output set
- global dependency rules tracked by `dust_driver`

Routing, state, i18n, and DB features can add cross-file dependency rules. Those
rules belong in the driver or plugin analysis layer, not in ad hoc cache reads.

## Edit Here When

- cache schema changes
- persisted fingerprints change
- build/check/watch invalidation changes need new cache data
- concurrent cache access behavior changes

See [`../dust_driver`](../dust_driver) for scheduling and invalidation policy.

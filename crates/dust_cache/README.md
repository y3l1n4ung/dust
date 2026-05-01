# dust_cache

Persistent build cache storage for Dust.

## Owns

- `.dart_tool/dust/build_cache_v1.json`
- cache schema versioning
- per-library fingerprints and cached analysis snapshots

## Used by

- `dust_driver`

## Edit here when

- cache schema changes
- new persisted fingerprints are added
- cached shared-analysis data changes shape

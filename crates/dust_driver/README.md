# dust_driver

Build orchestration for Dust.

## Owns

- `build`, `check`, `watch`, `clean`, and `doctor`
- cache usage
- shared-analysis collection and merge
- resolve/lower/emit scheduling
- progress reporting

## Used by

- `dust_cli`

## Edit here when

- pipeline stages change
- cache hit/miss behavior changes
- parallel scheduling or progress reporting changes
- new plugins need shared pipeline support

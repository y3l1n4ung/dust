# dust_resolver

Semantic resolution from parsed surfaces into Dust IR inputs.

## Owns

- annotation resolution
- constructor and field matching
- symbol catalog creation
- type and inheritance interpretation before lowering

## Used by

- `dust_driver`

## Edit here when

- parsed data needs semantic validation
- constructor rules change
- annotation arguments or type lookup rules change

# dust_parser_dart

Backend-neutral parser contracts and extracted Dart surface types.

## Owns

- `ParseBackend`
- `ParseOptions`
- `ParseResult`
- parsed class, enum, field, and annotation surfaces

## Used by

- `dust_parser_dart_ts`
- `dust_resolver`
- `dust_plugin_api`

## Edit here when

- the parse surface contract changes
- plugins or resolver need more parsed source detail
- backend-independent parsing behavior changes

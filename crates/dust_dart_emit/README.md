# dust_dart_emit

Shared Dart code rendering and formatting helpers for Dust plugins.

## Owns

- type rendering logic (`DartTypeRenderer`)
- rename rule implementation (PascalCase, lowerCamelCase, etc.)
- default type rendering strategies (dynamic vs Object?)

## Used by

- `dust_plugin_serde`
- `dust_state_plugin`
- `dust_http_client_plugin`

## Edit here when

- you need to change how types are rendered into Dart strings
- you need to add a new naming convention/rename rule
- you want to add shared Dart boilerplate used by multiple plugins

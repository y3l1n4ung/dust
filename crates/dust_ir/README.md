# dust_ir

Lowered Dust IR shared by plugins and the emitter.

## Owns

- class and enum IR
- type IR
- serde and derive metadata in normalized form

## Used by

- `dust_resolver`
- `dust_plugin_api`
- `dust_plugin_derive`
- `dust_plugin_serde`
- `dust_emitter`

## Edit here when

- generator features need new normalized structure
- resolver output must carry more semantic information
- plugins need a more stable shape than parsed source surfaces

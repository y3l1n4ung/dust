# dust_plugin_serde

Built-in plugin for JSON serialization and deserialization generation.

## Owns

- class `toJson()` and `_$TypeFromJson(...)` generation
- enum JSON mapping
- tagged sealed class dispatch and generated concrete variant classes
- serde field options such as rename, aliases, defaults, and skip rules
- `SerDeCodec` integration

## Used by

- `dust_driver`
- `dust_emitter`

## Edit here when

- serde validation rules change
- generated JSON helpers or support code change
- sealed class tag/content dispatch or generated variant class behavior changes
- enum or codec mapping behavior changes

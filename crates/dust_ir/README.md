# dust_ir

Shared intermediate representation for Dust.

`dust_ir` defines the normalized data model plugins consume after parsing,
resolution, and lowering. It is the boundary between Dart source facts and
feature generators.

## Owns

- `DartFileIr`, the file-level container used by plugins
- class, enum, member, type, annotation, and query metadata
- span references used for diagnostics
- structured feature configuration such as serde and SQL mapping options

## Design Rules

- Keep plugin-facing data normalized and deterministic.
- Add new IR fields only when a generator needs semantic information that cannot
  be derived reliably from existing data.
- Keep raw parser details out of plugin code.
- Preserve spans when changing lowered structures so diagnostics remain useful.

Plugins should treat IR values as read-only inputs. Resolver and driver lowering
are responsible for enriching parser facts before validation and generation.

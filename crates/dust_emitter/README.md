# dust_emitter

Final Dart file assembly for Dust generated output.

This crate merges plugin contributions into deterministic `.g.dart` files. It
owns layout, ordering, generated sections, and write-ready source text.

## Owns

- `DartWriter`
- generated section assembly
- contribution merging
- deterministic declaration ordering
- generated-file normalization
- final source buffers handed to `dust_driver`

## Design Rules

- Generated output must be deterministic.
- Prefer shared writer helpers over ad hoc string assembly.
- Do not rely on `dart format` as the generation strategy.
- Keep plugin-specific behavior inside plugins.
- Avoid unnecessary file changes when rendered output is unchanged.

The driver owns filesystem writes. The emitter owns the source text that should
be written.

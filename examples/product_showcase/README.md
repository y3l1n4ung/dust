# Product Showcase

Small real Dart package used to verify Dust end to end.

## What it covers

- `ToString()`, `Eq()`, and `CopyWith()` generation
- serde field options and enum support
- codec-backed serde fields
- analyzer-clean generated output
- runtime JSON round-trip tests

## Generate code

```bash
cargo run -p dust_cli -- build --root examples/product_showcase
```

## Validate

```bash
dart analyze
dart test
```

See [../../docs/developer.md](../../docs/developer.md) for the wider build
pipeline and [../stress_project/README.md](../stress_project/README.md) for the
large-scale perf fixture.

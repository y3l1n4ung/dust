# Release 0.1.0

## Scope

Release `0.1.0` for:

- Rust crates in the workspace
- `dust` CLI binary
- Dart packages:
  - `derive_annotation`
  - `derive_serde_annotation`
  - `dust_http_client_annotation`

## Pre-release Checks

Run from the repository root:

```sh
rtk cargo fmt --all --check
rtk cargo clippy --workspace --all-targets --all-features -- -D warnings
rtk cargo test --workspace --quiet
rtk cargo test -p dust_cli stress_project_release_build_benchmark -- --ignored --nocapture
rtk cargo run -q -p dust_cli -- build --root examples/product_showcase
rtk cargo run -q -p dust_cli -- build --root examples/stress_project
```

Run Dart checks:

```sh
cd examples/product_showcase
rtk dart analyze
rtk dart test

cd ../stress_project
rtk dart analyze
rtk dart test
```

Run package dry-runs:

```sh
cd packages/derive_annotation
rtk dart pub publish --dry-run

cd ../derive_serde_annotation
rtk dart pub publish --dry-run

cd ../dust_http_client_annotation
rtk dart pub publish --dry-run
```

Run Rust publish dry-runs in order:

```sh
rtk cargo publish --dry-run -p dust_text
rtk cargo publish --dry-run -p dust_diagnostics
rtk cargo publish --dry-run -p dust_ir
rtk cargo publish --dry-run -p dust_parser_dart
rtk cargo publish --dry-run -p dust_workspace
rtk cargo publish --dry-run -p dust_dart_emit
rtk cargo publish --dry-run -p dust_parser_dart_ts
rtk cargo publish --dry-run -p dust_plugin_api
rtk cargo publish --dry-run -p dust_cache
rtk cargo publish --dry-run -p dust_resolver
rtk cargo publish --dry-run -p dust_plugin_derive
rtk cargo publish --dry-run -p dust_plugin_serde
rtk cargo publish --dry-run -p dust_http_client_plugin
rtk cargo publish --dry-run -p dust_emitter
rtk cargo publish --dry-run -p dust_driver
rtk cargo publish --dry-run -p dust_cli
```

## Publish Order

Publish Rust crates to crates.io in this order:

1. `dust_text`
2. `dust_diagnostics`
3. `dust_ir`
4. `dust_parser_dart`
5. `dust_workspace`
6. `dust_dart_emit`
7. `dust_parser_dart_ts`
8. `dust_plugin_api`
9. `dust_cache`
10. `dust_resolver`
11. `dust_plugin_derive`
12. `dust_plugin_serde`
13. `dust_http_client_plugin`
14. `dust_emitter`
15. `dust_driver`
16. `dust_cli`

Publish Dart packages to pub.dev in this order:

1. `derive_annotation`
2. `derive_serde_annotation`
3. `dust_http_client_annotation`

## GitHub Release

After publishes succeed:

1. Create and push annotated tag `v0.1.0`.
2. Wait for `.github/workflows/release.yml` to attach:
   - `dust-x86_64-unknown-linux-gnu.tar.gz`
   - `dust-aarch64-unknown-linux-gnu.tar.gz`
   - `dust-x86_64-apple-darwin.tar.gz`
   - `dust-aarch64-apple-darwin.tar.gz`
   - `dust-x86_64-pc-windows-msvc.zip`
   - `dust-aarch64-pc-windows-msvc.zip`
   - `SHA256SUMS.txt`
3. Verify `install.sh` and `install.ps1` against the tagged release.

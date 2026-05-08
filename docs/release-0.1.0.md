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
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --quiet
cargo test -p dust_cli stress_project_release_build_benchmark -- --ignored --nocapture
cargo run -q -p dust_cli -- build --root examples/product_showcase
cargo run -q -p dust_cli -- build --root examples/stress_project
```

Run Dart checks:

```sh
cd examples/product_showcase
dart analyze
dart test

cd ../stress_project
dart analyze
dart test
```

Run package dry-runs:

```sh
cd packages/derive_annotation
dart pub publish --dry-run

cd ../derive_serde_annotation
dart pub publish --dry-run

cd ../dust_http_client_annotation
dart pub publish --dry-run
```

Run the Rust publish preflight:

```sh
python3 tmp/release_rust_crates.py
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

Run the actual Rust release with the helper:

```sh
python3 tmp/release_rust_crates.py --publish
```

If a publish succeeds and you need to resume later:

```sh
python3 tmp/release_rust_crates.py --publish --from dust_driver
```

Publish Dart packages to pub.dev in this order:

1. `derive_annotation`
2. `derive_serde_annotation`
3. `dust_http_client_annotation`

## GitHub Release

After publishes succeed:

1. Update the changelog and release notes manually:
   ```sh
   $EDITOR CHANGELOG.md
   $EDITOR release-notes/v0.1.0.md
   ```
2. Commit the release state on `main`.
3. Create and push annotated tag `v0.1.0`.
   ```sh
   git tag -a v0.1.0 -m "Dust v0.1.0"
   git push origin main
   git push origin v0.1.0
   ```
4. Wait for `.github/workflows/release.yml` to attach:
   - `dust-x86_64-unknown-linux-gnu.tar.gz`
   - `dust-aarch64-unknown-linux-gnu.tar.gz`
   - `dust-x86_64-apple-darwin.tar.gz`
   - `dust-aarch64-apple-darwin.tar.gz`
   - `dust-x86_64-pc-windows-msvc.zip`
   - `dust-aarch64-pc-windows-msvc.zip`
   - `SHA256SUMS.txt`
5. Verify `install.sh` and `install.ps1` against the tagged release.

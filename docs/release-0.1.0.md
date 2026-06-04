# Release 0.1.0

## Scope

Release `0.1.0` for:

- `dust` CLI binary
- Dart packages:
  - `dust_dart`
  - `dust_flutter`
  - `dust_db_sqlite3`

## Pre-release Checks

Run from the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --quiet
cargo test -p dust_cli benchmark_project_release_build_benchmark -- --ignored --nocapture
cargo run -q -p dust_cli -- build --root examples/product_showcase
cargo run -q -p dust_cli -- build --root examples/benchmark_project
```

Run Dart checks:

```sh
cd examples/product_showcase
flutter analyze
flutter test

cd ../benchmark_project
flutter analyze
flutter test
```

Run package dry-runs:

```sh
cd packages/dust_dart
dart pub publish --dry-run

cd ../dust_flutter
dart pub publish --dry-run

cd ../dust_db_sqlite3
dart pub publish --dry-run
```

## Publish Order

Skip crates.io for Rust crates. Release the `dust` CLI from GitHub binary
artifacts and installers.

Publish Dart packages to pub.dev in this order:

1. `dust_dart`
2. `dust_flutter`
3. `dust_db_sqlite3`

## GitHub Release

After Dart package publishes and Rust binary checks succeed:

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

## Install Verification

Verify the release with both supported installation paths:

```sh
cargo install --git https://github.com/y3l1n4ung/dust --tag v0.1.0 dust_cli
dust --help
dust --version
```

Then test the binary installer against the tagged GitHub release:

```sh
curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/v0.1.0/install.sh | bash
dust --help
dust --version
```

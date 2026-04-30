# Release Automation Plan

## Goal

Make Dust releases mostly mechanical:

- one version bump flow
- one changelog generation flow
- one tag push that triggers GitHub release assets
- optional automated publish to crates.io and pub.dev once metadata is ready

## Recommended Model

- [ ] keep one shared version `X.Y.Z` across the Rust workspace and both public
  Dart annotation packages
- [ ] keep `vX.Y.Z` as the primary release tag for GitHub releases and binary
  assets
- [ ] require Conventional Commits so changelog generation stays useful
- [ ] generate changelog content before tagging, not after publishing

This is the simplest release model for Dust today because the Rust workspace,
`derive_annotation`, and `derive_serde_annotation` are still moving together.

If Dart packages need independent release cadence later, split pub.dev tags to
package-prefixed forms such as `derive_annotation-vX.Y.Z`.

## Current Gaps

### Metadata

- [ ] update Dart package `homepage`, `repository`, and `issue_tracker` to
  `https://github.com/y3l1n4ung/dust`
- [ ] remove `publish_to: none` from `packages/derive_serde_annotation/pubspec.yaml`
- [ ] decide whether all Rust crates are public crates.io packages or internal
  workspace crates
- [ ] if some Rust crates stay internal, mark them `publish = false`

### Cargo Publish Readiness

- [ ] decide cargo strategy:
  - publish the full Rust workspace to crates.io
  - or keep crates private and ship only the `dust` binary from GitHub Releases
- [ ] if publishing Rust crates, replace pure path-only internal dependencies
  with publishable workspace dependency declarations
- [ ] add crates.io metadata for every published crate before enabling CI

### Changelog

- [ ] add top-level `CHANGELOG.md`
- [ ] add `cliff.toml` with commit grouping for `feat`, `fix`, `docs`, `refactor`,
  `perf`, and breaking changes
- [ ] fail CI or release prep when commit history is too unconventional to
  build a clean changelog

## Tooling Plan

### Rust Versioning

- [ ] add `cargo-release`
- [ ] add workspace `release.toml`
- [ ] use `cargo release <level> --workspace` as the Rust version bump entrypoint
- [ ] let `cargo-release` own:
  - workspace version bumps
  - release commit creation
  - `vX.Y.Z` tag creation
  - push

### Dart Versioning

- [ ] add one small script to sync Dart package versions to the Rust workspace
  version
- [ ] call that script from `cargo-release` pre-release hooks or replacements
- [ ] keep `derive_annotation` and `derive_serde_annotation` on the same version
  until there is a real need to split them

### Changelog Generation

- [ ] generate `CHANGELOG.md` with `git-cliff`
- [ ] run changelog generation before the release commit
- [ ] include the generated changelog in the version bump commit
- [ ] use the matching changelog section as GitHub Release notes

## CI / Workflow Plan

### GitHub Release Assets

- [x] keep `.github/workflows/release.yml` for cross-platform binary assets
- [ ] replace the placeholder GitHub release notes with generated changelog text
- [ ] upload `CHANGELOG.md` section or generated notes for the tagged version

### pub.dev Publish

- [ ] publish the first version of each Dart package manually if it is not
  already on pub.dev
- [ ] enable automated publishing on pub.dev for each package
- [ ] use GitHub Actions OIDC for pub.dev publishing
- [ ] require a protected GitHub environment such as `pub.dev`
- [ ] decide tag pattern:
  - keep one shared `v{{version}}` tag if both packages always release together
  - or switch to per-package tags if package release cadence diverges

### crates.io Publish

- [ ] only add crates.io publish automation after the Rust crate strategy is
  explicit
- [ ] if publishing, store `CARGO_REGISTRY_TOKEN` in GitHub Actions secrets
- [ ] publish crates after tests pass and before GitHub Release notes are finalized

## Release Flow

### Phase 1: Prep

- [ ] run `cargo fmt --all --check`
- [ ] run `cargo test --workspace --quiet`
- [ ] run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] run `dart analyze` and `dart test` for public Dart packages
- [ ] run showcase generation, analyze, and tests

### Phase 2: Cut Version

- [ ] choose semver level: `patch`, `minor`, or `major`
- [ ] generate changelog for the release
- [ ] bump Rust workspace version
- [ ] sync Dart package versions
- [ ] commit version and changelog changes
- [ ] create and push tag `vX.Y.Z`

### Phase 3: Publish

- [ ] let GitHub Actions build release binaries from the tag
- [ ] let GitHub Actions publish Dart packages if enabled
- [ ] let GitHub Actions publish crates.io packages if enabled
- [ ] verify GitHub Release assets, pub.dev versions, and crates.io versions

## Proposed Implementation Order

- [ ] step 1: clean package metadata and decide crates.io scope
- [ ] step 2: add `CHANGELOG.md` and `git-cliff`
- [ ] step 3: add `release.toml` and version-sync script
- [ ] step 4: replace GitHub release notes with changelog-driven notes
- [ ] step 5: enable pub.dev automated publishing
- [ ] step 6: enable crates.io publishing only if Rust crate metadata is ready

## Non-Goals For The First Automation Pass

- [ ] independent versioning per internal Rust crate
- [ ] independent versioning per Dart package
- [ ] prerelease channel automation
- [ ] automatic version inference from commit history without maintainer review

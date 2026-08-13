# CLI and Package Compatibility

Dust generated code depends on runtime APIs from the Dart and Flutter packages.
Keep the CLI and packages in a supported compatibility row.

The machine-readable source of truth is
[`compatibility/dust-cli-packages.json`](../../compatibility/dust-cli-packages.json).
CLI diagnostics, release checks, and docs should read from that contract instead
of duplicating version rules.

Run `dust doctor` to see the active CLI version, each Dust runtime package found
in `package_config.json`, the supported range, and whether the package is used
by the workspace source.

## Dust CLI 0.1.4

| Dust CLI | `dust_dart` | `dust_flutter` | `dust_db_sqlite3` |
| :--- | :--- | :--- | :--- |
| `0.1.4` | `>=0.1.3 <0.2.0` | `>=0.1.4 <0.2.0` | `>=0.1.3 <0.2.0` |

## When Versions Do Not Match

- If the CLI is too old, install a newer Dust CLI release before generating.
- If a Dust package is too old, upgrade that package in `pubspec.yaml`.
- If a Dust package is newer than the CLI supports, upgrade the CLI first or pin
  the package back to the supported range.
- After changing Dust versions, run `dust clean`, `dust build`, and `dust check`
  so generated files match the active runtime packages.

> [!TIP]
> AI agents should check this file before editing Dust package constraints. Do
> not bump only one Dust package past the supported CLI row.

## Maintainer Rule

When a publishable package changes runtime source, public API, dependency
constraints, runtime behavior, or generated-code compatibility, bump that
package's `pubspec.yaml` version and update its `CHANGELOG.md` in the same PR.
If generated code needs the new package API, update
`compatibility/dust-cli-packages.json` too.

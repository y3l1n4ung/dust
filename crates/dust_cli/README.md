# dust_cli

The `dust` command-line interface.

You focus on product. We focus on performance.

Dust CLI is the binary users run for generation. It should keep app-facing
commands stable while engine, cache, and generated-code quality improve behind
the same command surface.

## Distribution

Dust is not published to crates.io. Do not use `cargo install dust_cli` unless
the project later changes to a crates.io release model. Use GitHub release
artifacts, installers, or a local workspace build.

## Owns

- command parsing
- terminal output and compact summaries
- banner/help rendering
- CLI integration and perf tests

## Used by

- end users
- CI

## Edit here when

- command surface changes
- output UX changes
- perf smoke tests or command-level validation change

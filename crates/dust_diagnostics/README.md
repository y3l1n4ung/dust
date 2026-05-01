# dust_diagnostics

Shared diagnostics model for Dust.

## Owns

- diagnostic severity and structure
- message text and labeled spans
- reusable error reporting contracts across crates

## Used by

- parser, resolver, plugins, emitter, workspace, and driver

## Edit here when

- a crate needs richer diagnostic data
- output formatting or severity rules change
- source span reporting needs to improve

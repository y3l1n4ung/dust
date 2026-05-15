# Usage Guides

This section is the canonical user-facing guide for Dust.

If you are new to the project, read these pages in order:

1. [Derive](./derive.md)
2. [Serde](./serde.md)
3. [HttpClient](./http.md)
4. [State Management](./state.md)

## Package Map

Use this package when you want generated object helpers:

- `derive_annotation`

Add this package when you also want JSON serialization:

- `derive_serde_annotation`

Add these packages when you want generated Dio clients:

- `dust_http_client_annotation`
- `dio`

Add this package for state management (integrated in prototype):

- `dust_state_annotation` (Coming soon)

## Quick Start

Install the CLI:

```bash
curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.sh | bash
```

If you have Rust installed and want to install directly from the repository:

```bash
cargo install --git https://github.com/y3l1n4ung/dust dust_cli
```

Add the Dart packages you need:

```yaml
dependencies:
  derive_annotation: ^0.1.0
  derive_serde_annotation: ^0.1.0
  dust_http_client_annotation: ^0.1.0
  dio: ^5.9.2
```

Fetch packages and generate code:

```bash
dart pub get
dust build
```

## Runnable Reference

The guides in this directory point at the live example package in
[../../examples/product_showcase](../../examples/product_showcase/README.md).

If you are working inside this repository, build it with:

```bash
cargo run -p dust_cli -- build --root examples/product_showcase
```

## Related Docs

- [Developer guide](../developer.md)
- [Plugin guide](../plugin-guide.md)
- [Roadmap](../roadmap/README.md)

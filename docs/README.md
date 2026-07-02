# Documentation

**You focus on product. We focus on performance.**

## Our Promise

- Stable public APIs for features marked stable.
- Improvements should change generated code, engine internals, or runtime
  internals before they require changes to handwritten product code.
- Features marked beta may still receive API refinements before stabilization.
- Docs and examples should show the stable authoring API first, then generated
  output only when it clarifies behavior.

Dust has two documentation tracks:

- user-facing usage guides in [usage](./usage/README.md)
- contributor and architecture docs in this directory

## Documentation Types

Use the four documentation roles below when adding or reviewing pages:

| Type | Reader intent | Current pages |
| :--- | :--- | :--- |
| Tutorial | Learn by following a strict path to a first successful result. | [README quick start](../README.md#quick-start), [usage quick start](./usage/README.md#quick-start), [product showcase](../examples/product_showcase/README.md) |
| How-to guide | Solve one named task after learning the basics. | [Derive](./usage/derive.md), [Serde](./usage/serde.md), [Validation](./usage/validation.md), [HttpClient](./usage/http.md), [State Management](./usage/state.md), [Routing](./usage/routing.md), [i18n](./usage/i18n.md), [Dust DB](./usage/db.md) |
| Reference | Look up exact facts: packages, flags, config, generated files, schemas, and release steps. | [Package installation](./usage/README.md#package-installation), [release runbook](./release-0.1.0.md), [dust_dart](../packages/dust_dart/README.md), [dust_flutter](../packages/dust_flutter/README.md), [dust_db_sqlite3](../packages/dust_db_sqlite3/README.md) |
| Explanation | Understand design choices, constraints, and trade-offs. | [Developer guide](./developer.md), [Plugin guide](./plugin-guide.md), [State management design](./state-management-design.md) |

## Usage Guides

- [Usage overview](./usage/README.md)
- [Derive guide](./usage/derive.md)
- [Serde guide](./usage/serde.md)
- [Validation guide](./usage/validation.md)
- [HttpClient guide](./usage/http.md)
- [State Management guide](./usage/state.md)
- [Routing guide](./usage/routing.md)
- [i18n guide](./usage/i18n.md)
- [Dust DB guide](./usage/db.md)

These pages are backed by the runnable example package in
[../examples/product_showcase](../examples/product_showcase/README.md).

## Contributor Docs

- [Developer guide](./developer.md)
- [Plugin guide](./plugin-guide.md)
- [State management design](./state-management-design.md)
- [Release runbook](./release-0.1.0.md)
- [Roadmap and milestones](https://github.com/y3l1n4ung/dust/milestones)

## Example Packages

- [Product showcase](../examples/product_showcase/README.md)
- [Benchmark project](../examples/benchmark_project/README.md)
- [Shopping app](../examples/shopping_app/README.md)

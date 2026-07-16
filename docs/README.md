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

## Documentation Roles

Use the four documentation roles below when adding or reviewing pages. Keep a
page's primary role clear; link to another page instead of mixing tutorial,
how-to, reference, and explanation content in one place.

| Type | Reader intent | Pages |
| :--- | :--- | :--- |
| Tutorial | Learn by following a strict path to a first successful result. | [README quick start](../README.md#quick-start), [product showcase](../examples/product_showcase/README.md), [shopping app](../examples/shopping_app/README.md) |
| How-to guide | Solve one named task after learning the basics. | [Derive](./usage/derive.md), [Serde](./usage/serde.md), [Validation](./usage/validation.md), [HttpClient](./usage/http.md), [State Management](./usage/state.md), [Routing](./usage/routing.md), [i18n](./usage/i18n.md), [Database](./usage/db.md) |
| Reference | Look up exact facts: packages, flags, config, generated files, schemas, and release steps. | [Package installation](./usage/README.md#package-installation), [release runbook](./release-0.1.0.md), [dust_dart](../packages/dust_dart/README.md), [dust_flutter](../packages/dust_flutter/README.md), [dust_db_sqlite3](../packages/dust_db_sqlite3/README.md) |
| Explanation | Understand design choices, constraints, and trade-offs. | [Developer guide](./developer.md), [Plugin guide](./plugin-guide.md), [State management design](./state-management-design.md) |

## Page Audit

| Page | Primary role | Notes |
| :--- | :--- | :--- |
| [Root README](../README.md) | Tutorial | Landing page, release installation, and first successful app generation. |
| [Usage overview](./usage/README.md) | Reference | Guide order, package map, and links to runnable examples. |
| [Derive](./usage/derive.md) | How-to + reference | Generate data-class helpers; trait table is the reference section. |
| [Serde](./usage/serde.md) | How-to + reference | Generate JSON codecs; option tables are the reference sections. |
| [Validation](./usage/validation.md) | How-to + reference | Add Dart model validation and Flutter-only form validators; rule tables are reference sections. |
| [HttpClient](./usage/http.md) | How-to + reference | Generate Dio clients; annotation tables are reference sections. |
| [State Management](./usage/state.md) | How-to + reference | Build sync and async ViewModels; generated API and rules are reference sections. |
| [Routing](./usage/routing.md) | How-to + reference | Configure typed Navigator 2.0 routing; parameters and guards are reference sections. |
| [i18n](./usage/i18n.md) | How-to + reference | Configure ARB build, runtime lookup, and overrides. |
| [Database](./usage/db.md) | How-to + reference | Configure SQLx-style sqlite3 validation; pipeline split is reference. |
| [Developer guide](./developer.md) | Explanation | Architecture, pipeline, and engineering trade-offs. |
| [Plugin guide](./plugin-guide.md) | How-to + explanation | Plugin implementation steps with design constraints. |
| [State management design](./state-management-design.md) | Explanation | State API rationale and hardening notes. |
| [Release runbook](./release-0.1.0.md) | Reference | Release checklist, publish policy, and install verification. |
| [Product showcase](../examples/product_showcase/README.md) | Tutorial | Runnable example backing the usage guides. |
| [Benchmark project](../examples/benchmark_project/README.md) | Reference | Scale/performance fixture. |
| [Shopping app](../examples/shopping_app/README.md) | Tutorial | End-to-end Flutter proof app. |

## Usage Guides

- [Usage overview](./usage/README.md)
- [Generate data classes](./usage/derive.md)
- [Generate JSON serialization](./usage/serde.md)
- [Add validation](./usage/validation.md)
- [Generate HTTP clients](./usage/http.md)
- [Build ViewModels](./usage/state.md)
- [Configure typed routing](./usage/routing.md)
- [Add i18n](./usage/i18n.md)
- [Validate SQLite queries](./usage/db.md)

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

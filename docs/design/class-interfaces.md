# Class Interfaces in IR

Status: proposal. This document scopes adding an `implements` clause to Dust's
parsed surface and IR, and replacing the structural interface detection that
several plugins do today.

For the build pipeline and crate map, see the [Developer Guide](../developer.md).
The feature that first needed this is described in the
[Server Plugin Design](server-plugin.md), which was reworked to *not* need it.

## Problem

`ClassIr` records `superclass_name: Option<String>` and `is_interface`, but
nothing about `implements` (`crates/dust_ir/src/class.rs`). The word does not
appear anywhere in `dust_parser_dart` or `dust_parser_dart_ts`, so the
information is lost at parse time, not at lowering.

Plugins that need to know "does this class implement interface X" therefore
guess from method shape. Serde detects `toJson` that way
(`crates/dust_plugin_serde/src/analysis.rs`), and the server design does the
same for `IntoResponse` and `Validatable`. Structural detection has three
costs:

| Cost | Example |
| :--- | :--- |
| False positives | a class with an unrelated `Response intoResponse()` is treated as a response type |
| False negatives | an interface method reached through a mixin or superclass is invisible |
| No diagnostics | Dust cannot say "you declared `implements Extractable` but the method shape is wrong"; the analyzer reports it later, against generated code |

## Non-goals

- Full type resolution. Dust records the written names, not resolved
  declarations, matching how `superclass_name` already works.
- Transitive interface closure. A class implementing `B`, where `B` implements
  `A`, records `B` only. Consumers that need the closure walk it themselves
  using the workspace fact below.
- Generic argument semantics. `Extractable<AuthUser>` is recorded with its
  argument as written; Dust does not check variance or substitution.

## Surface

```rust
pub struct ParsedClassSurface {
    // ...
    pub superclass_name: Option<String>,
    /// Interfaces named in an `implements` clause, in source order.
    pub interfaces: Vec<ParsedTypeRef>,
    /// Mixins named in a `with` clause, in source order.
    pub mixins: Vec<ParsedTypeRef>,
}
```

`mixins` rides along because it is the same grammar neighborhood and the same
one-line extraction, and because `with _$TodoController` is how every generated
mixin is applied. A plugin that wants to verify its own mixin was applied
currently cannot.

`ParsedTypeRef` carries the written name plus type arguments as text, which is
what distinguishes `Extractable<AuthUser>` from `Extractable<Todo>`:

```rust
pub struct ParsedTypeRef {
    pub name: String,
    pub type_arguments: Vec<String>,
    pub span: TextRange,
}
```

`TypeIr` already exists for lowered types; whether `interfaces` should carry
`TypeIr` instead of a new struct is the one open question worth settling before
implementation, and it depends on whether `TypeIr` can be built without
resolution.

## Where the change lands

| Crate | Change |
| :--- | :--- |
| `crates/dust_parser_dart` | add the fields to `ParsedClassSurface` (`src/surface.rs`) |
| `crates/dust_parser_dart_ts` | extract them in `src/classes/class_decl.rs`, beside the existing `superclass` field read |
| `crates/dust_ir` | add `interfaces` and `mixins` to `ClassIr` (`src/class.rs`) |
| `crates/dust_driver` | copy them through `lower.rs`, near the existing `superclass_name` hand-off |
| `crates/dust_plugin_serde` | replace structural `toJson` detection, keeping it as a fallback |

The tree-sitter side is the only part with real unknowns: `class_declaration`
exposes `superclass` as a named field today, and in tree-sitter-dart the
`implements` clause is a child of that same `superclass` node rather than a
sibling. Confirming the node kinds against the vendored grammar is the first
task, not an assumption to build on.

## Detection policy

Adding the clause does not mean structural detection goes away. The rule:

1. A declared interface is authoritative. `implements IntoResponse` means the
   class is a response type; a wrong method shape is now a Dust diagnostic
   naming the class and the expected signature.
2. Structural detection stays as a fallback, because a class can satisfy a
   contract through a mixin or a superclass without naming it, and because
   removing it would break every project that relies on today's behavior.
3. Where the two disagree, declared but structurally wrong, the declaration
   wins and Dust emits the diagnostic. Silent fallback would hide the typo the
   feature exists to catch.

## What it unlocks

| Consumer | Today | With interfaces |
| :--- | :--- | :--- |
| serde `toJson` | structural, false positives accepted | declared, with a diagnostic |
| server `IntoResponse`, `Validatable` | structural | declared |
| server `Extractable` | irrelevant, the extractor type is named at the call site | bare-type extractor parameters become possible |
| generated mixins | unverifiable | Dust can check `with _$Foo` was applied and diagnose when it was not |

The server plugin's bare-type extractor form is the largest of these and is
described under
[Rejected: bare-type binding](server-plugin.md#rejected-bare-type-binding). It
needs one more thing beyond this change: a `dust_server.extractables.v1`
workspace fact mapping value type to extractor type, following the JSON-in-facts
convention the route plugin already uses
(`crates/dust_route_plugin/src/plugin/analysis.rs`), plus an ambiguity
diagnostic when two extractors produce one value type. That belongs to the
server plugin, not here.

## Phases

| Phase | Scope | Exit criteria |
| :--- | :--- | :--- |
| 0 | Confirm the tree-sitter node kinds for `implements` and `with` against the vendored grammar; add parser fixtures covering `implements A`, `implements A<T>, B`, `with M implements A`, and `extends S with M implements A`. | Fixtures assert the extracted names. |
| 1 | `ParsedClassSurface` fields, tree-sitter extraction, `ClassIr` fields, lowering hand-off. | A class's interfaces survive to IR, asserted in `dust_driver` lowering tests. |
| 2 | Serde adopts the declared clause with structural fallback and a mismatch diagnostic. | Existing serde fixtures unchanged; new fixture for the mismatch diagnostic. |
| 3 | Mixin verification: diagnose a `@Derive`d class missing its generated `with _$Name`. | Fixture for the missing-mixin diagnostic. |

Phases 2 and 3 are independent of each other and either can be dropped without
stranding Phase 1.

## Risks

- **Cache invalidation.** Interfaces become part of the parsed surface, so a
  change to an `implements` clause must invalidate dependent files. Any
  consumer building cross-file facts from interfaces takes a cache dependency,
  the same way the route plugin's facts do.
- **Fallback drift.** Keeping structural detection alongside the declared
  clause means two code paths to test. Every consumer that adopts the clause
  needs fixtures for both paths, not just the happy one.
- **Grammar coupling.** The extraction is tied to tree-sitter-dart node kinds.
  A grammar bump can silently return no interfaces, which would degrade to
  today's behavior rather than fail loudly, so the Phase 0 fixtures are the
  regression net.

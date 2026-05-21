# Dust State Management Feature Checklist

## Legend

- `[x]` done and verified.
- `[~]` partial/prototype only.
- `[ ]` not done yet.

## Product Contract

- [x] Decide final package name and crate name.
- [x] Define public annotation API for `@ViewModel`, dependency injection, listeners, and side effects.
- [x] Keep runtime API small and Flutter-native: `ValueNotifier`, `InheritedModel`, `BuildContext` extensions.
- [x] No external state package dependency.
- [x] Document generated file ownership and import contract.
- [x] Define migration story from manually-written prototype `.g.dart` files to Rust-generated output.

## Prototype Quality

- [x] `examples/state_management_prototype` passes `flutter analyze` with no warnings.
- [x] `examples/state_management_prototype` passes `flutter test`.
- [x] `examples/state_management_prototype` passes `flutter build web`.
- [x] Extract shared runtime primitives into `packages/dust_state`.
- [x] Check generated/runtime scopes dispose owned ViewModels exactly once.
- [x] Listener API moved to effect streams and covered by generated listener tests.
- [x] Check `read` never registers dependency in generated contract.
- [x] Manual prototype uses typed aspect enums for accessed fields.
- [x] Runtime `ViewModelBase.init()` is idempotent and tested.
- [x] Check errors/loading states are deterministic for overlapping async refreshes.
- [x] Check repository injection failure has clear diagnostics/runtime error.
- [x] Add rebuild-count tests for aspect precision.
- [x] Add listener side-effect tests.
- [ ] Add navigation integration tests.

## Runtime Design

- [x] Extract shared runtime primitives into package-level contract.
- [x] Define `ViewModelBase<TState, TArgs>` responsibilities.
- [x] Define generated scope lifecycle protocol with owned and `.value` modes.
- [x] Define generated proxy/aspect protocol.
- [x] Define generated listener protocol as effect-stream listener.
- [x] Support sync and async ViewModel actions.
- [x] Support cancellation/stale async protection.
- [x] Support observer hooks for logging/devtools.
- [x] Support test-friendly deterministic initialization.

## Rust Generator Plan

- [x] Add annotation package plan.
- [x] Add Rust crate plan: `crates/dust_state_plugin`.
- [~] Parser supports `@ViewModel`; `@State` is not implemented.
- [x] Workspace analysis collects ViewModel and state field facts.
- [x] Validation rejects missing state constructors.
- [ ] Validation rejects mutable/non-final state fields.
- [ ] Validation rejects unsupported dependency injection types.
- [x] Emission generates base class, scope, proxy, listener, context extensions, and state aspects.
- [x] Generated code is stable and `dart format` clean for smoke coverage.
- [~] Add golden tests for generated output; emission coverage now checks fieldless state, observer exclusion, multi-output, and scope debug names.
- [ ] Add driver tests for stale/check/watch/clean behavior.

## Docs

- [x] Replace placeholder `docs/usage/state.md` content with production-facing examples.
- [x] Update `docs/roadmap/state-management.md` to match final package/crate names.
- [x] Add prototype README contract: manual `.g.dart` files are generator targets.
- [x] Add examples for watch/read/listen, DI, async loading, errors, and observer.
- [x] Add anti-patterns: side effects in build, broad watches, mutable state.

## Release Gate

- [x] Rust tests pass for state plugin.
- [x] Flutter prototype analyze/test/build pass.
- [ ] Generated state code has golden snapshots.
- [x] Docs match actual API.

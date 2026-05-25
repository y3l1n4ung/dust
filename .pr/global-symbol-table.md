# Implementation Plan: Measured String Interning

## Background & Motivation
The engine now has sub-second warm builds and roughly 1s invalidated rebuilds on the 5,000-file stress project. Remaining parser/lowering overhead may still come from repeated identifier/import URI allocations, but this must be proven before changing public IR ownership.

## Scope & Impact
Do not global-intern blindly.

The safe target is a per-build interner for hot identifiers and import URIs during parse/lower. The IR may later hold compact symbol keys or `Arc<str>`, but only after benchmarks show that the conversion improves 10k+ file workloads without complicating plugin APIs.

## Current Benchmark Gate

The ignored release perf test now measures:
- cold 5k-file build
- warm 5k-file build
- tool-hash-invalidated 5k-file rebuild

```bash
cargo test -p dust_cli stress_project_release_build_benchmark -- --ignored --nocapture
```

## Proposed Phased Implementation

### Phase 1: Instrument First
1. Capture allocation/CPU profiles for invalidated derive-heavy stress builds.
2. Identify whether parse, lowering, resolver lookup, or emission owns the remaining cost.
3. Keep benchmark thresholds in `crates/dust_cli/tests/perf_tests.rs` as the regression gate.

### Phase 2: Per-Build Interner Prototype
1. Add an internal `StringInterner` owned by one build batch.
2. Intern only high-frequency identifiers/import URIs during parse/lower.
3. Keep plugin APIs string-compatible until measurements justify a broader migration.

### Phase 3: IR Representation Decision
1. Prefer `Arc<str>` for low-risk sharing if pointer-sized symbol keys add too much API friction.
2. Use compact symbol keys only for fields proven hot by profiling.
3. Preserve deterministic output by resolving symbols at emit boundaries.

### Phase 4: Global Symbol Table Only If Needed
1. Add global declaration indexing only if inheritance/resolution remains a measured bottleneck.
2. Keep it separate from string interning so the two optimizations can be benchmarked independently.

## Verification
- Run focused parser/lower/resolver tests after each prototype.
- Run `cargo test -p dust_cli stress_project_release_build_benchmark -- --ignored --nocapture`.
- Compare cold, warm, and invalidated rebuild timings before merging.

# HttpClient Specification: DX & Production-Grade Features

This document defines advanced features aimed at maximizing Developer Experience (DX) and ensuring the generated code is "production-hardened" while remaining a thin, high-performance wrapper over `package:dio`.

## 1. High-Tier Developer Experience (DX)

### A. Proactive Compile-Time Diagnostics
The Dust engine must provide rich, actionable error messages during the build phase to prevent runtime failures.
- **Serialization Validation**: Error if a `@Body` parameter or return type `T` lacks a visible `toJson()` or `fromJson()`.
- **Placeholder Matching**: Error if a `{id}` placeholder in a path has no matching `@Path('id')` parameter.
- **Redundant Annotations**: Warning if multiple HTTP verbs are applied to the same method.
- **Type Safety**: Warning if `dynamic` is used as a return type when a model is expected.

### B. Automated Docstring Preservation
The generator must copy Dartdocs from the user's interface to the generated implementation.
- **Rationale**: Ensures IDE tooltips, "Go to Definition," and generated documentation (like `dartdoc`) remain useful and informative.

### C. Functional "Result" Pattern
To improve DX and reduce `try-catch` boilerplate, Dust can optionally generate return types using a `Result<T, ApiException>` pattern.
- **Implementation**: Uses a standard sealed class hierarchy for errors, allowing users to use `switch` or functional mapping (`fold`) on API results.

---

## 2. Production-Grade Code Quality

### A. Zero-Allocation Path Interpolation
- **Spec**: Use `StringBuffer` or optimized string interpolation for path variables to minimize transient allocations in high-frequency API calls.
- **Encoding**: Ensure all `@Path` and `@Query` values are handled with `Uri.encodeComponent` only where necessary to prevent double-encoding.

### B. Analyzer-Clean Output
- **Requirement**: The generated `.g.dart` file must pass `dart analyze` with zero hints, warnings, or lints (including standard `package:lints/recommended.yaml`).
- **Formatting**: Generated code should be deterministic and follow `dart format` conventions.

### C. Performance Instrumentation (Telemetry)
Instead of custom interceptors, the generated code can optionally include "Hooks" or specific `extra` metadata to help standard `Dio` interceptors track:
- **Parse Duration**: Time spent in `Isolate.run` vs. the main thread.
- **Payload Size**: Content-length tracking for both request and response.
- **Thread Context**: Identifying if the request was offloaded to an isolate.

### D. Modern Isolate Efficiency
- **Rule**: When using `DustParseThread.isolate`, ensure the top-level decode function is generated with `static` or top-level scope to avoid closure capture overhead.
- **Optimization**: For small primitive returns (e.g., `Future<bool>`), bypass `Isolate.run` even if configured for isolates to avoid unnecessary thread-hop overhead.

---

## 3. Thin Wrapper Philosophy
Dust does **not** reimplement features provided by `Dio`.
- **Interceptors**: Users should use standard `Dio` interceptors for Auth, Logging, and Caching.
- **Adapters**: Users should use standard `Dio` adapters for Certificate Pinning and custom HTTP protocols.
- **Consistency**: Generated code follows the `Dio` internal `compose` and `fetch` patterns to ensure 100% compatibility with the existing `Dio` ecosystem.

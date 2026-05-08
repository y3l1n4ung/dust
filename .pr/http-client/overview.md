# HttpClient Specification: Overview

## 1. Goal
Provide a type-safe, high-performance, and production-ready HTTP networking layer for Dart and Flutter applications. This specification defines the behavior of the Dust code generator and the runtime architecture built on top of `package:dio`.

## 2. Core Philosophy
- **Thin Wrapper**: Dust generates the "plumbing" to call `Dio` but stays out of the way of `Dio`'s native features like interceptors and adapters.
- **Type Safety**: Eliminate `dynamic` maps in application logic by moving serialization to the generated layer.
- **Zero Boilerplate**: Users define interfaces; Dust implements the networking logic.
- **Production Built-in**: First-class support for isolates, cancellation, and progress tracking.
- **Analyzer Compliance**: Generated code is 100% analyzer-clean and idiomatic.

## 3. High-Level Architecture

The system is split into three layers:

### A. Annotation Layer (`dust_http_client_annotation`)
A lightweight Dart package containing only metadata.

### B. Generation Layer (`dust_http_client_plugin`)
A Rust-based plugin that parses interfaces and emits optimized Dart code.

### C. Runtime Layer (`package:dio`)
Dust relies entirely on `Dio` for the heavy lifting. The user provides a configured `Dio` instance, maintaining full control over interceptors, timeouts, and security.

## 4. Model Contract (Serialization)

For Dust to generate type-safe networking code, all models used as request bodies or response types must follow a standard serialization contract:

1.  **Decoding (Response)**: The model `T` must have a factory constructor or static method: `T.fromJson(Map<String, dynamic> json)`.
2.  **Encoding (Request)**: The model must have a method: `Map<String, dynamic> toJson()`.

This contract is compatible with `package:dust_serde`, `package:json_serializable`, hand-written models, and `freezed`.

## 5. Naming Conventions

- **Generated Class**: For an interface named `UserApi`, the generator produces `_$UserApi`.
- **Part File**: The user must include `part 'user_api.g.dart';` in their source file.
- **Internal Helpers**: Top-level private helpers for isolate decoding are prefixed with `_$decode`.

## 6. Master Index
- [Annotations Spec](./annotations.md)
- [Parsing & Isolate Spec](./parsing.md)
- [Runtime & Error Handling Spec](./runtime.md)
- [Feature Examples](./examples.md)
- [Testing & Faking Spec](./testing.md)
- [DX & Production-Grade Spec](./dx-production.md)


# HttpClient Master Specification

This directory contains the complete technical specification for the Dust HttpClient generation engine and its production-ready runtime architecture.

## Section Specs
1.  **[Overview](./http-client/overview.md)**: Goals, high-level architecture, and "Thin Wrapper" philosophy.
2.  **[Annotations Spec](./http-client/annotations.md)**: Detailed documentation for all supported Dart annotations.
3.  **[Parsing & Isolate Spec](./http-client/parsing.md)**: Strategies for JSON serialization and background execution using modern Dart isolates.
4.  **[Runtime & Error Handling Spec](./http-client/runtime.md)**: Best practices for Dio configuration and domain error mapping.
5.  **[Feature Examples](./http-client/examples.md)**: Real-world scenarios including CRUD, heavy data parsing, and file uploads.
6.  **[Testing & Faking Spec](./http-client/testing.md)**: Strategies for manual mocking and auto-generated test suites.
7.  **[DX & Production-Grade Spec](./http-client/dx-production.md)**: Advanced diagnostics, generated code quality, and performance instrumentation.

## Future Roadmap (V2+)
- **Streaming**: Support for `Stream<T>` response types.
- **WebSockets**: Integrated support for bidirectional streaming.
- **Custom Converters**: `@HttpConverter()` for manual control over individual field or method serialization.
- **Retry Policy Annotations**: Declarative retry strategies directly on the API interface.

---
*Status: Design Phase (Complete)*

/// Stable functional primitives for Dust runtimes and generated code.
///
/// Import this library when app code wants Dust-owned `Option`, `Result`, or
/// `Unit` types without taking a dependency on an external functional package.
/// These are Dust's own types rather than a re-export of `fpdart` or `dartz`.
/// Generated code returns them, so they cannot be optional, and a runtime that
/// pulled in a functional package would put that package in the dependency
/// graph of every project that runs `dust build`. Owning three small types
/// costs less than that.
///
/// The contract is stable within a major version, and `Option` is still
/// growing: see the tracking issue for the methods it does not have yet.
///
/// ```dart
/// import 'package:dust_dart/fp.dart';
///
/// final nickname = Some<String?>('John');
/// final saved = Ok<Unit, String>(unit);
/// ```
library;

export 'src/fp/option.dart';
export 'src/fp/result.dart';
export 'src/fp/unit.dart';

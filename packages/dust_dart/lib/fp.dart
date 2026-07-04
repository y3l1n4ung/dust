/// Beta functional primitives for Dust runtimes and generated code.
///
/// Import this library when app code wants Dust-owned `Option`, `Result`, or
/// `Unit` types without taking a dependency on an external functional package.
/// The generated-code contract is available, but app-facing ergonomics can
/// still be refined before this library is marked stable.
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

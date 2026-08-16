/// Annotations: what Dust reads to generate handlers.
///
/// Import this in a file that only declares controllers or handler functions
/// and leaves the wiring to generated code.
///
/// ```dart
/// import 'package:dust_server/annotations.dart';
///
/// @Controller('/todos')
/// class TodoController with _$TodoController { ... }
/// ```
///
/// Everything here is also exported from `package:dust_server/server.dart`.
library;

export 'src/annotations/annotations.dart';

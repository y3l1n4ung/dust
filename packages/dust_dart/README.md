# dust_dart

Dart-only runtime and annotations for Dust generated code.

## Public surfaces

- `package:dust_dart/core.dart`: shared core primitives such as `Result`, `Ok`, `Err`, `Unit`, and `unit`.
- `package:dust_dart/derive.dart`: derive annotations and marker traits.
- `package:dust_dart/serde.dart`: JSON/serde annotations and runtime helpers.
- `package:dust_dart/http.dart`: HTTP client annotations.
- `package:dust_dart/db.dart`: SQLx-style DB annotations and runtime contracts.
- `package:dust_dart/dust_dart.dart`: convenience export for all Dart-only APIs.

Dust owns its functional primitives. Do not add `fpdart`, `dartz`, or another external functional package for generated-code `Result` handling.

## Result

```dart
import 'package:dust_dart/core.dart';

Result<int, String> parseCount(String text) {
  final value = int.tryParse(text);
  return value == null ? const Err('invalid count') : Ok(value);
}

final label = parseCount('42').match(
  ok: (value) => 'count=$value',
  err: (error) => error,
);
```

## DB compatibility

`package:dust_dart/db.dart` re-exports `Result`, `Ok`, `Err`, `Unit`, and `unit` so generated DAO code can use:

```dart
Future<Result<UserRow?, SqlxError>> findById(int id);
```

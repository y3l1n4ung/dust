# dust_dart

Dart-only runtime and annotations for Dust generated code.

You focus on product. We focus on performance.

## Our Promise

- Stable Dart authoring APIs for data classes, JSON, validation, and HTTP
  client generation.
- DB APIs are 50% stable and may still be refined before stabilization.
- Generated code and runtime helpers can improve without forcing app-code
  rewrites.
- No external functional dependency for core generated-code contracts.

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

`Executor` is the SQLx-style execution contract used by generated DAO code.
`Pool`, `Connection`, and `Transaction` implementations can all be passed to
DAO factories and query helpers.

`Row` is a driver-agnostic interface. Driver packages own concrete adapters
such as `Sqlite3Row`, while generated `FromRow` mappers stay driver-blind:

```dart
extension UserRowFromRow on UserRow {
  static UserRow fromRow(Row row) {
    return UserRow(id: row.read<int>('id'));
  }
}
```

Use driver-specific escape hatches only when needed, for example
`Sqlite3Executor.database` from `package:dust_db_sqlite3`.

## Validation

This package keeps its public runtime surface analyzer-clean, fully documented,
and fully covered. The package analysis options enable
`public_member_api_docs`, so every public annotation, runtime type, constructor,
field, and method needs Dartdoc.

Run the package gate after changing runtime code or annotations:

```bash
dart format --set-exit-if-changed packages/dust_dart/lib packages/dust_dart/test
dart analyze packages/dust_dart
dart --enable-asserts test --coverage=packages/dust_dart/coverage packages/dust_dart/test
dart run coverage:format_coverage --check-ignore \
  --packages=.dart_tool/package_config.json \
  --report-on=packages/dust_dart/lib \
  --in=packages/dust_dart/coverage \
  --out=packages/dust_dart/coverage/lcov.info \
  --lcov
awk 'BEGIN{lf=lh=0} /^LF:/{v=$0; sub("LF:","",v); lf+=v} /^LH:/{v=$0; sub("LH:","",v); lh+=v} END{printf("TOTAL LH=%d LF=%d %.2f%%\n", lh, lf, (lf?100*lh/lf:100)); exit(lh==lf && lf>0 ? 0 : 1)}' packages/dust_dart/coverage/lcov.info
```

The `SerDeCodec` abstract interface constructor is excluded with
`coverage:ignore-line` because package users can implement the interface but
cannot extend it from outside the defining library.

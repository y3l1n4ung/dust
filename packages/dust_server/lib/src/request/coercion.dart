import 'package:dust_dart/fp.dart';

import '../response/rejection.dart';

/// Reifies [X] so nullable targets can be compared, since `T == String?` is not
/// valid syntax.
Type typeOf<X>() => X;

/// Converts one raw string from a path segment, query value, header, or form
/// field into [T].
///
/// Supported targets are `String`, `int`, `double`, `num`, `bool`, `Uri`,
/// `DateTime`, and `BigInt`, plus their nullable forms. Anything else is a
/// programming error rather than a request error, so it throws instead of
/// rejecting.
///
/// `bool` accepts `true`, `false`, `1`, and `0`, and treats a bare flag with an
/// empty value as `true`, matching how query flags are usually written.
///
/// Integers are decimal only, and are not padded with whitespace. Dart's
/// parsers are lenient in two ways a request should not be: `int.tryParse`
/// reads `0x10` as 16, and every numeric parser skips surrounding whitespace,
/// so `?id=0x10`, `?id=%2010`, and `?id=10` would all name the same record.
/// Anything keyed on the raw text — a cache key, a rate-limit bucket, a dedup
/// check — then disagrees with the handler about which request this was. A
/// request carries a bare decimal number, so anything else is a 400.
Result<T, Rejection> coerce<T>(String raw, {required String source}) {
  Rejection bad(String type) =>
      Rejection.badRequest('$source is not a valid $type');

  if (T == String || T == typeOf<String?>()) {
    return Ok(raw as T);
  }
  if (T == int || T == typeOf<int?>()) {
    final value = _padded(raw) ? null : int.tryParse(raw, radix: 10);
    return value == null ? Err(bad('integer')) : Ok(value as T);
  }
  if (T == double || T == typeOf<double?>()) {
    final value = _padded(raw) ? null : double.tryParse(raw);
    return value == null ? Err(bad('number')) : Ok(value as T);
  }
  if (T == num || T == typeOf<num?>()) {
    final value = _padded(raw) ? null : num.tryParse(raw);
    return value == null ? Err(bad('number')) : Ok(value as T);
  }
  if (T == bool || T == typeOf<bool?>()) {
    final value = switch (raw.toLowerCase()) {
      'true' || '1' || '' => true,
      'false' || '0' => false,
      _ => null,
    };
    return value == null ? Err(bad('boolean')) : Ok(value as T);
  }
  if (T == BigInt || T == typeOf<BigInt?>()) {
    final value = _padded(raw) ? null : BigInt.tryParse(raw, radix: 10);
    return value == null ? Err(bad('integer')) : Ok(value as T);
  }
  if (T == DateTime || T == typeOf<DateTime?>()) {
    final value = DateTime.tryParse(raw);
    return value == null ? Err(bad('date-time')) : Ok(value as T);
  }
  if (T == Uri || T == typeOf<Uri?>()) {
    final value = Uri.tryParse(raw);
    return value == null ? Err(bad('URI')) : Ok(value as T);
  }

  throw ArgumentError('dust_server cannot coerce a request value to $T');
}

/// Whether [raw] carries whitespace a numeric parser would skip.
///
/// Dart's `int.tryParse`, `double.tryParse`, `num.tryParse`, and
/// `BigInt.tryParse` all trim before parsing, so `" 5"`, `"5 "`, and `"5"`
/// produce the same number from three different requests.
bool _padded(String raw) => raw.trim().length != raw.length;

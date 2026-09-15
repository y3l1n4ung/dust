/// Result of an operation that can either succeed with [T] or fail with [E].
///
/// Use `Ok` for successful values and `Err` for failures. Chain validation or
/// conversion steps with `andThen`, and recover failures with `orElse`.
///
/// ```dart
/// Result<int, String> parseCount(String text) {
///   final value = int.tryParse(text);
///   return value == null ? const Err('invalid count') : Ok(value);
/// }
///
/// Result<int, String> requirePositive(int value) {
///   return value > 0 ? Ok(value) : const Err('count must be positive');
/// }
///
/// final count = parseCount('42').andThen(requirePositive);
/// ```
sealed class Result<T, E> {
  /// Creates one result value.
  ///
  /// Subclasses are usually created with `Ok(value)` or `Err(error)`.
  ///
  /// ```dart
  /// const result = Ok<int, String>(42);
  /// ```
  const Result();

  /// Whether this result is [Ok].
  ///
  /// ```dart
  /// const result = Ok<int, String>(42);
  /// final succeeded = result.isOk; // true
  /// ```
  bool get isOk;

  /// Whether this result is [Err].
  ///
  /// ```dart
  /// const result = Err<int, String>('invalid');
  /// final failed = result.isErr; // true
  /// ```
  bool get isErr => !isOk;

  /// Maps a successful value and leaves errors unchanged.
  ///
  /// ```dart
  /// final doubled = const Ok<int, String>(2).map((value) => value * 2);
  /// final unchanged = const Err<int, String>('missing')
  ///     .map((value) => value * 2);
  /// ```
  Result<R, E> map<R>(R Function(T value) mapper);

  /// Maps an error value and leaves successful values unchanged.
  ///
  /// ```dart
  /// final readable = const Err<int, String>('bad')
  ///     .mapErr((error) => 'error: $error');
  /// final unchanged = const Ok<int, String>(42)
  ///     .mapErr((error) => 'error: $error');
  /// ```
  Result<T, F> mapErr<F>(F Function(E error) mapper);

  /// Pattern matches this result.
  ///
  /// ```dart
  /// final label = const Ok<int, String>(42).match(
  ///   ok: (value) => 'count=$value',
  ///   err: (error) => error,
  /// );
  /// ```
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  });
}

/// Successful operation result.
///
/// ```dart
/// const result = Ok<int, String>(42);
/// ```
final class Ok<T, E> extends Result<T, E> {
  /// Creates one successful result.
  ///
  /// ```dart
  /// const result = Ok<int, String>(42);
  /// ```
  const Ok(this.value);

  /// Successful value.
  ///
  /// ```dart
  /// const result = Ok<int, String>(42);
  /// final value = result.value; // 42
  /// ```
  final T value;

  @override
  bool get isOk => true;

  @override
  Result<R, E> map<R>(R Function(T value) mapper) {
    return Ok<R, E>(mapper(value));
  }

  @override
  Result<T, F> mapErr<F>(F Function(E error) mapper) {
    return Ok<T, F>(value);
  }

  @override
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  }) {
    return ok(value);
  }

  @override
  bool operator ==(Object other) {
    return other is Ok<Object?, Object?> && other.value == value;
  }

  @override
  int get hashCode => Object.hash(Ok, value);

  @override
  String toString() => 'Ok($value)';
}

/// Failed operation result.
///
/// ```dart
/// const result = Err<int, String>('invalid count');
/// ```
final class Err<T, E> extends Result<T, E> {
  /// Creates one failed result.
  ///
  /// ```dart
  /// const result = Err<int, String>('invalid count');
  /// ```
  const Err(this.error);

  /// Error value.
  ///
  /// ```dart
  /// const result = Err<int, String>('invalid count');
  /// final error = result.error;
  /// ```
  final E error;

  @override
  bool get isOk => false;

  @override
  Result<R, E> map<R>(R Function(T value) mapper) {
    return Err<R, E>(error);
  }

  @override
  Result<T, F> mapErr<F>(F Function(E error) mapper) {
    return Err<T, F>(mapper(error));
  }

  @override
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  }) {
    return err(error);
  }

  @override
  bool operator ==(Object other) {
    return other is Err<Object?, Object?> && other.error == error;
  }

  @override
  int get hashCode => Object.hash(Err, error);

  @override
  String toString() => 'Err($error)';
}

/// Chaining and extraction on [Result].
///
/// An extension rather than members of [Result], so the types come from the
/// static type at the call site. As members they were checked against the
/// type a value was built with, and an `Ok<int, NotFound>` held in a
/// `Result<int, AppError>` threw a `TypeError` when chained with a step that
/// fails with another `AppError`.
extension ResultChain<T, E> on Result<T, E> {
  /// Chains another result-producing operation when this result is successful.
  ///
  /// Use this when the next step can also fail.
  ///
  /// ```dart
  /// Result<int, String> parseCount(String text) {
  ///   final value = int.tryParse(text);
  ///   return value == null ? const Err('invalid') : Ok(value);
  /// }
  ///
  /// Result<int, String> requirePositive(int value) {
  ///   return value > 0 ? Ok(value) : const Err('must be positive');
  /// }
  ///
  /// final parsed = parseCount('2').andThen(requirePositive);
  /// final failed = parseCount('-1').andThen(requirePositive);
  /// ```
  Result<R, E> andThen<R>(Result<R, E> Function(T value) next) {
    return switch (this) {
      Ok<T, E>(:final value) => next(value),
      Err<T, E>(:final error) => Err<R, E>(error),
    };
  }

  /// Chains another result-producing operation when this result failed.
  ///
  /// Use this for fallback reads or error recovery that can still fail.
  ///
  /// ```dart
  /// Result<int, String> readPrimary() => const Err('cache miss');
  /// Result<int, String> readFallback(String error) => const Ok(42);
  ///
  /// final value = readPrimary().orElse(readFallback);
  /// ```
  Result<T, F> orElse<F>(Result<T, F> Function(E error) next) {
    return switch (this) {
      Ok<T, E>(:final value) => Ok<T, F>(value),
      Err<T, E>(:final error) => next(error),
    };
  }

  /// Returns the successful value, or [fallback] when this result failed.
  ///
  /// ```dart
  /// final count = const Err<int, String>('invalid').unwrapOr(0);
  /// final existing = const Ok<int, String>(7).unwrapOr(0);
  /// ```
  T unwrapOr(T fallback) {
    return switch (this) {
      Ok<T, E>(:final value) => value,
      Err<T, E>() => fallback,
    };
  }

  /// Returns the successful value, or computes one from the error.
  ///
  /// [fallback] is not called for an [Ok].
  ///
  /// ```dart
  /// final count = const Err<int, String>('invalid')
  ///     .unwrapOrElse((error) => error.length);
  /// final existing = const Ok<int, String>(7)
  ///     .unwrapOrElse((error) => error.length);
  /// ```
  T unwrapOrElse(T Function(E error) fallback) {
    return switch (this) {
      Ok<T, E>(:final value) => value,
      Err<T, E>(:final error) => fallback(error),
    };
  }
}

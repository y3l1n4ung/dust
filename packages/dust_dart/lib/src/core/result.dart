/// Result of an operation that can either succeed with [T] or fail with [E].
sealed class Result<T, E> {
  /// Creates one result value.
  const Result();

  /// Whether this result is [Ok].
  bool get isOk;

  /// Whether this result is [Err].
  bool get isErr => !isOk;

  /// Maps a successful value and leaves errors unchanged.
  Result<R, E> map<R>(R Function(T value) mapper);

  /// Maps an error value and leaves successful values unchanged.
  Result<T, F> mapErr<F>(F Function(E error) mapper);

  /// Chains another result-producing operation when this result is successful.
  Result<R, E> andThen<R>(Result<R, E> Function(T value) next);

  /// Chains another result-producing operation when this result failed.
  Result<T, F> orElse<F>(Result<T, F> Function(E error) next);

  /// Returns the successful value, or [fallback] when this result failed.
  T unwrapOr(T fallback);

  /// Returns the successful value, or computes one from the error.
  T unwrapOrElse(T Function(E error) fallback);

  /// Pattern matches this result.
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  });
}

/// Successful operation result.
final class Ok<T, E> extends Result<T, E> {
  /// Creates one successful result.
  const Ok(this.value);

  /// Successful value.
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
  Result<R, E> andThen<R>(Result<R, E> Function(T value) next) {
    return next(value);
  }

  @override
  Result<T, F> orElse<F>(Result<T, F> Function(E error) next) {
    return Ok<T, F>(value);
  }

  @override
  T unwrapOr(T fallback) => value;

  @override
  T unwrapOrElse(T Function(E error) fallback) => value;

  @override
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  }) {
    return ok(value);
  }

  @override
  bool operator ==(Object other) {
    return other is Ok<T, E> && other.value == value;
  }

  @override
  int get hashCode => Object.hash(Ok, value);

  @override
  String toString() => 'Ok($value)';
}

/// Failed operation result.
final class Err<T, E> extends Result<T, E> {
  /// Creates one failed result.
  const Err(this.error);

  /// Error value.
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
  Result<R, E> andThen<R>(Result<R, E> Function(T value) next) {
    return Err<R, E>(error);
  }

  @override
  Result<T, F> orElse<F>(Result<T, F> Function(E error) next) {
    return next(error);
  }

  @override
  T unwrapOr(T fallback) => fallback;

  @override
  T unwrapOrElse(T Function(E error) fallback) => fallback(error);

  @override
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  }) {
    return err(error);
  }

  @override
  bool operator ==(Object other) {
    return other is Err<T, E> && other.error == error;
  }

  @override
  int get hashCode => Object.hash(Err, error);

  @override
  String toString() => 'Err($error)';
}

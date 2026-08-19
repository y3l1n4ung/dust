import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../response/rejection.dart';

/// Builds a [T] from a request, or rejects it.
///
/// Custom extractors implement this and are named at the call site with
/// `@Extract(MyExtractor)`. Configuration goes in the constructor, and a
/// preconfigured variant is a subclass with a zero-argument `const`
/// constructor, which is what the generated code calls.
///
/// ```dart
/// final class BearerAuth implements FromRequestParts<AuthUser> {
///   const BearerAuth({this.scope});
///
///   final String? scope;
///
///   @override
///   Future<Result<AuthUser, Rejection>> extract(Request request) async { ... }
/// }
///
/// final class TodosWrite extends BearerAuth {
///   const TodosWrite() : super(scope: 'todos:write');
/// }
/// ```
abstract interface class FromRequestParts<T> {
  /// Produces the value, or the rejection that stops the handler.
  Future<Result<T, Rejection>> extract(Request request);
}

/// Marks an extractor that reads the request body.
///
/// At most one of these runs per handler, and it has to run last, because the
/// body can only be read once. Dust checks both for built-ins when it
/// generates; the marker is here so hand-written compositions can check too.
abstract interface class FromRequest<T> implements FromRequestParts<T> {}

/// Turns a client-error rejection into [None].
///
/// Wrap any extractor with this to let a handler declare `Option<T>` and treat
/// a missing or malformed value as absent.
///
/// A 5xx rejection passes through instead. Those mean something on the server
/// is wrong, such as middleware that never wrote the context value being read,
/// and hiding them would make a broken deployment look like an anonymous
/// request.
///
/// ```dart
/// const viewer = OptionalExtractable(BearerAuth());
/// ```
final class OptionalExtractable<T> implements FromRequestParts<Option<T>> {
  /// Wraps [inner].
  const OptionalExtractable(this.inner);

  /// The extractor whose client-error rejection is swallowed.
  final FromRequestParts<T> inner;

  @override
  Future<Result<Option<T>, Rejection>> extract(Request request) async {
    final outcome = await inner.extract(request);
    return switch (outcome) {
      Ok(:final value) => Ok(Some(value)),
      Err(:final error) when error.status >= 500 => Err(error),
      Err() => Ok(None<T>()),
    };
  }
}

/// Hands the rejection to the handler instead of short-circuiting.
///
/// Wrap any extractor with this to let a handler declare
/// `Result<T, Rejection>` and decide what to do about the failure itself.
///
/// ```dart
/// const outcome = FallibleExtractable(BearerAuth());
/// ```
final class FallibleExtractable<T>
    implements FromRequestParts<Result<T, Rejection>> {
  /// Wraps [inner].
  const FallibleExtractable(this.inner);

  /// The extractor whose outcome is passed through.
  final FromRequestParts<T> inner;

  @override
  Future<Result<Result<T, Rejection>, Rejection>> extract(
    Request request,
  ) async {
    return Ok(await inner.extract(request));
  }
}

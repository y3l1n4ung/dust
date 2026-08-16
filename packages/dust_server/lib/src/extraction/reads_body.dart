import 'extractable.dart';
import 'validated.dart';

/// Whether [extractor] ends up reading the request body.
///
/// [FromRequest] marks the extractors that do, and at most one of them may run
/// per handler because a body can only be read once. The wrappers break that
/// check on their own: `OptionalExtractable(JsonExtractable(...))` is a
/// `FromRequestParts`, so a type test on the outside says the body is never
/// touched while the extractor underneath drains it.
///
/// This looks through the wrappers instead, so a composition check sees what
/// will actually happen.
///
/// ```dart
/// readsRequestBody(const OptionalExtractable(RawBodyExtractable())); // true
/// readsRequestBody(const PathExtractable<String>('id')); // false
/// ```
bool readsRequestBody(FromRequestParts<Object?> extractor) {
  return switch (extractor) {
    FromRequest() => true,
    OptionalExtractable(:final inner) => readsRequestBody(inner),
    FallibleExtractable(:final inner) => readsRequestBody(inner),
    ValidatedExtractable(:final inner) => readsRequestBody(inner),
    _ => false,
  };
}

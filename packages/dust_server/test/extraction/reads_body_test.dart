import 'package:dust_dart/derive.dart';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// A body can be read once, so a composition check has to know which
/// extractors will reach for it. The `FromRequest` marker says so directly,
/// but a wrapper hides it: `OptionalExtractable(JsonExtractable(...))` is a
/// `FromRequestParts`, and a type test on the outside would report that the
/// body is never touched while the extractor underneath drains it.
///
/// These pin that the answer follows the extractor that actually runs.

final class _Payload implements Validatable {
  const _Payload();

  static _Payload deserialize(Map<String, Object?> json) => const _Payload();

  @override
  ValidationResult validate() => const Valid();

  @override
  void validateOrThrow() {}
}

/// Neither a body extractor nor one of the wrappers.
final class _Custom implements FromRequestParts<String> {
  const _Custom();

  @override
  Future<Result<String, Rejection>> extract(Request request) async =>
      const Ok('x');
}

void main() {
  const jsonBody = JsonExtractable<_Payload>(_Payload.deserialize);

  group('an unwrapped extractor', () {
    test('reads the body when it is marked FromRequest', () {
      expect(readsRequestBody(jsonBody), isTrue);
    });

    test('reads the body for every built-in body extractor', () {
      expect(
        [
          readsRequestBody(const RawBodyExtractable()),
          readsRequestBody(const TextBodyExtractable()),
          readsRequestBody(const StreamBodyExtractable()),
          readsRequestBody(const FormExtractable()),
          readsRequestBody(const MultipartExtractable()),
          readsRequestBody(const JsonListExtractable<_Payload>(
            _Payload.deserialize,
          )),
        ],
        everyElement(isTrue),
      );
    });

    test('does not read the body for a parts extractor', () {
      expect(readsRequestBody(const PathExtractable<String>('id')), isFalse);
    });

    test('does not read the body for an extractor of its own', () {
      expect(readsRequestBody(const _Custom()), isFalse);
    });
  });

  group('a wrapped extractor', () {
    test('reports the body through Optional', () {
      expect(readsRequestBody(const OptionalExtractable(jsonBody)), isTrue);
    });

    test('reports the body through Fallible', () {
      expect(readsRequestBody(const FallibleExtractable(jsonBody)), isTrue);
    });

    test('reports the body through Validated', () {
      expect(readsRequestBody(const ValidatedExtractable(jsonBody)), isTrue);
    });

    test('reports the body through the shortcut spellings', () {
      expect(
        [
          readsRequestBody(optional(body<_Payload>(_Payload.deserialize))),
          readsRequestBody(fallible(body<_Payload>(_Payload.deserialize))),
          readsRequestBody(valid(body<_Payload>(_Payload.deserialize))),
        ],
        everyElement(isTrue),
      );
    });

    test('looks through more than one layer', () {
      expect(
        readsRequestBody(
          const OptionalExtractable(FallibleExtractable(jsonBody)),
        ),
        isTrue,
      );
    });

    test('stays false when the extractor underneath reads no body', () {
      expect(
        readsRequestBody(
          const OptionalExtractable(PathExtractable<String>('id')),
        ),
        isFalse,
      );
    });

    test('stays false through several layers over a parts extractor', () {
      expect(
        readsRequestBody(
          const FallibleExtractable(
            OptionalExtractable(PathExtractable<String>('id')),
          ),
        ),
        isFalse,
      );
    });
  });
}

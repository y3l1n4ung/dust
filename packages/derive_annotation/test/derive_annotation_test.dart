import 'package:derive_annotation/derive_annotation.dart';
import 'package:test/test.dart';

final class Serialize extends DeriveTrait {
  const Serialize();
}

final class SerDe extends DeriveConfig {
  const SerDe();
}

void main() {
  group('Derive trait markers', () {
    test('core markers are derive traits', () {
      const traits = <DeriveTrait>[
        Debug(),
        Clone(),
        CopyWith(),
        PartialEq(),
        Eq(),
        Hash(),
      ];

      expect(traits, hasLength(6));
      expect(traits, everyElement(isA<DeriveMeta>()));
    });

    test('Derive stores a typed trait list', () {
      const annotation = Derive([
        Debug(),
        PartialEq(),
        Hash(),
        CopyWith(),
      ]);

      expect(annotation.traits, hasLength(4));
      expect(annotation.traits[0], isA<Debug>());
      expect(annotation.traits[1], isA<PartialEq>());
      expect(annotation.traits[2], isA<Hash>());
      expect(annotation.traits[3], isA<CopyWith>());
    });
  });

  group('Extension points', () {
    test('future trait packages can extend DeriveTrait', () {
      const derive = Derive([Serialize()]);

      expect(derive.traits.single, isA<Serialize>());
      expect(derive.traits.single, isA<DeriveTrait>());
    });

    test('future config packages can extend DeriveConfig', () {
      const config = SerDe();

      expect(config, isA<DeriveConfig>());
      expect(config, isA<DeriveMeta>());
    });
  });

  test('re-exports collection helpers for future generated deep equality', () {
    const equality = DeepCollectionEquality();

    expect(
      equality.equals(<Object?>[
        'a',
        <Object?>['b', 1]
      ], <Object?>[
        'a',
        <Object?>['b', 1]
      ]),
      isTrue,
    );
  });
}

import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test('copyWith deep copies nested collection fields without replacement', () {
    final original = NestedBundle(
      groups: [
        ['sale', 'featured'],
        ['clearance'],
      ],
      metrics: {
        'yangon': [1, 2],
        'mandalay': [3],
      },
    );

    final cloned = original.copyWith();
    original.groups[0].add('vip');
    original.metrics['yangon']!.add(99);

    expect(cloned.groups[0], ['sale', 'featured']);
    expect(cloned.metrics['yangon'], [1, 2]);
    expect(cloned, isNot(same(original)));
    expect(cloned.groups, isNot(same(original.groups)));
    expect(cloned.metrics, isNot(same(original.metrics)));
  });

  test('copyWith deep copies nested collection fields', () {
    final original = NestedBundle(
      groups: [
        ['sale', 'featured'],
      ],
      metrics: {
        'yangon': [1, 2],
      },
    );
    final replacementGroups = [
      ['vip'],
    ];
    final replacementMetrics = {
      'mandalay': [4, 5],
    };

    final copied = original.copyWith(
      groups: replacementGroups,
      metrics: replacementMetrics,
    );
    replacementGroups[0].add('late');
    replacementMetrics['mandalay']!.add(99);

    expect(copied.groups[0], ['vip']);
    expect(copied.metrics['mandalay'], [4, 5]);
    expect(copied.groups, isNot(same(replacementGroups)));
    expect(copied.metrics, isNot(same(replacementMetrics)));

    final retained = original.copyWith();
    original.groups[0].add('clearance');
    original.metrics['yangon']!.add(7);

    expect(retained.groups[0], ['sale', 'featured']);
    expect(retained.metrics['yangon'], [1, 2]);
  });

  test('copyWith keeps typed params and clears nullable fields explicitly', () {
    final original = OptionalNote(
      id: 'note-1',
      note: 'seasonal',
      aliases: ['sale', 'featured'],
    );

    final renamed = original.copyWith(id: 'note-2');
    expect(renamed.id, 'note-2');
    expect(renamed.note, 'seasonal');
    expect(renamed.aliases, ['sale', 'featured']);

    final replaced = original.copyWith(aliases: const Some(['clearance']));
    expect(replaced.aliases, ['clearance']);
    expect(replaced.aliases, isNot(same(original.aliases)));

    final cleared = original.copyWith(
      note: const Some(null),
      aliases: const Some(null),
    );
    expect(cleared.note, isNull);
    expect(cleared.aliases, isNull);

    final retained = original.copyWith();
    expect(retained.note, 'seasonal');
    expect(retained.aliases, ['sale', 'featured']);
  });

  test(
    'generated derive features work for extends and existing mixin chains',
    () {
      const price = Price(currency: 'USD', cents: 4500, tags: ['vip']);
      const left = FeaturedProduct(
        sku: 'featured-1',
        price: price,
        tags: {'vip', 'seasonal'},
        archived: false,
      );
      const right = FeaturedProduct(
        sku: 'featured-1',
        price: Price(currency: 'USD', cents: 4500, tags: ['vip']),
        tags: {'seasonal', 'vip'},
        archived: false,
      );

      expect(left.auditLabel(), 'audited');
      expect(left, equals(right));
      expect(left.hashCode, equals(right.hashCode));
      expect(left.copyWith(), equals(left));

      final updated = left.copyWith(archived: true);
      expect(updated.archived, isTrue);
      expect(updated.sku, left.sku);
      expect(updated.tags, left.tags);
    },
  );
}

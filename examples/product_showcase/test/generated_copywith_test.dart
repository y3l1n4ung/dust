import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test('copyWith keeps collection field references without replacement', () {
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

    final copied = original.copyWith();
    original.groups[0].add('vip');
    original.metrics['yangon']!.add(99);

    expect(copied, isNot(same(original)));
    expect(copied.groups, same(original.groups));
    expect(copied.metrics, same(original.metrics));
    expect(copied.groups[0], ['sale', 'featured', 'vip']);
    expect(copied.metrics['yangon'], [1, 2, 99]);
  });

  test('copyWith stores replacement collections by identity', () {
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

    expect(copied.groups, same(replacementGroups));
    expect(copied.metrics, same(replacementMetrics));
    expect(copied.groups[0], ['vip', 'late']);
    expect(copied.metrics['mandalay'], [4, 5, 99]);

    final retained = original.copyWith();
    original.groups[0].add('clearance');
    original.metrics['yangon']!.add(7);

    expect(retained.groups, same(original.groups));
    expect(retained.metrics, same(original.metrics));
    expect(retained.groups[0], ['sale', 'featured', 'clearance']);
    expect(retained.metrics['yangon'], [1, 2, 7]);
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

    final replacementAliases = ['clearance'];
    final replaced = original.copyWith(aliases: replacementAliases);
    expect(replaced.aliases, ['clearance']);
    expect(replaced.aliases, same(replacementAliases));

    final cleared = original.copyWith(note: null, aliases: null);
    expect(cleared.note, isNull);
    expect(cleared.aliases, isNull);

    final retained = original.copyWith();
    expect(retained.note, 'seasonal');
    expect(retained.aliases, ['sale', 'featured']);
    expect(retained.aliases, same(original.aliases));
  });

  test('copyWith exposes chained helpers for nested models', () {
    const original = Profile(
      name: 'Jane',
      nickname: 'Jay',
      address: Address(city: 'Paris', line1: '1 Main Street'),
      mailingAddress: Address(city: 'Berlin', line1: '2 Postal Road'),
    );

    final renamed = original.copyWith(name: 'John');
    expect(renamed.name, 'John');
    expect(renamed.address, same(original.address));
    expect(renamed.mailingAddress, same(original.mailingAddress));

    final cleared = original.copyWith(nickname: null, mailingAddress: null);
    expect(cleared.nickname, isNull);
    expect(cleared.mailingAddress, isNull);

    final moved = original.copyWith.address(city: 'London');
    expect(moved.address.city, 'London');
    expect(moved.address.line1, '1 Main Street');
    expect(moved.mailingAddress, same(original.mailingAddress));

    final movedNullable = original.copyWith.mailingAddress?.call(city: 'Rome');
    expect(movedNullable, isNotNull);
    expect(movedNullable!.mailingAddress!.city, 'Rome');

    const withoutMailing = Profile(
      name: 'Jane',
      address: Address(city: 'Paris', line1: '1 Main Street'),
    );
    expect(withoutMailing.copyWith.mailingAddress, isNull);
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

import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test('generated derive features work across multiple models', () {
    const price = Price(
      currency: 'USD',
      cents: 1999,
      tags: ['featured', 'sale'],
    );
    const samePrice = Price(
      currency: 'USD',
      cents: 1999,
      tags: ['featured', 'sale'],
    );
    expect(price, equals(samePrice));
    expect(price.hashCode, equals(samePrice.hashCode));
    expect(price.toString(), contains('Price('));

    const category = Category(
      id: 'cat-1',
      title: 'Shoes',
      labels: {'summer', 'sale'},
    );
    final updatedCategory = category.copyWith(title: 'Sneakers');
    expect(updatedCategory.title, 'Sneakers');
    expect(updatedCategory.labels, category.labels);

    final product = Product(
      sku: 'sku-1',
      title: 'Runner',
      price: price,
      categories: const [category],
      attributes: const {'color': 'black', 'size': '42'},
      featured: true,
    );
    final sameProduct = Product(
      sku: 'sku-1',
      title: 'Runner',
      price: samePrice,
      categories: const [
        Category(id: 'cat-1', title: 'Shoes', labels: {'sale', 'summer'}),
      ],
      attributes: const {'size': '42', 'color': 'black'},
      featured: true,
    );

    expect(product, equals(sameProduct));
    expect(product.hashCode, equals(sameProduct.hashCode));
    expect(product.copyWith(), equals(product));
    expect(product.copyWith(featured: false).featured, isFalse);

    final inventory = const [
      InventoryEntry(productSku: 'sku-1', warehouse: 'yangon-a', quantity: 10),
      InventoryEntry(productSku: 'sku-1', warehouse: 'yangon-b', quantity: 5),
    ];
    final catalog = Catalog(
      id: 'catalog-1',
      products: [product],
      categoryById: const {'cat-1': category},
      featuredSkus: const {'sku-1'},
      inventory: inventory,
    );
    final sameCatalog = Catalog(
      id: 'catalog-1',
      products: [sameProduct],
      categoryById: const {
        'cat-1': Category(
          id: 'cat-1',
          title: 'Shoes',
          labels: {'summer', 'sale'},
        ),
      },
      featuredSkus: const {'sku-1'},
      inventory: const [
        InventoryEntry(
          productSku: 'sku-1',
          warehouse: 'yangon-a',
          quantity: 10,
        ),
        InventoryEntry(productSku: 'sku-1', warehouse: 'yangon-b', quantity: 5),
      ],
    );

    expect(catalog, equals(sameCatalog));
    expect(catalog.hashCode, equals(sameCatalog.hashCode));
    expect(catalog.copyWith(id: 'catalog-2').id, 'catalog-2');
    expect(catalog.toString(), contains('Catalog('));
  });

  test('copyWith deep clones nested Dust models', () {
    final price = Price(currency: 'USD', cents: 1999, tags: ['featured', 'sale']);
    final category = Category(
      id: 'cat-1',
      title: 'Shoes',
      labels: {'summer', 'sale'},
    );
    final product = Product(
      sku: 'sku-1',
      title: 'Runner',
      price: price,
      categories: [category],
      attributes: {'color': 'black'},
      featured: true,
    );

    final cloned = product.copyWith();
    price.tags.add('vip');
    category.labels.add('clearance');

    expect(cloned.price, isNot(same(price)));
    expect(cloned.categories[0], isNot(same(category)));
    expect(cloned.price.tags, ['featured', 'sale']);
    expect(cloned.categories[0].labels, {'summer', 'sale'});

    final copied = product.copyWith(featured: false);
    price.tags.add('limited');
    category.labels.add('outlet');

    expect(copied.featured, isFalse);
    expect(copied.price, isNot(same(price)));
    expect(copied.categories[0], isNot(same(category)));
    expect(copied.price.tags, ['featured', 'sale', 'vip']);
    expect(copied.categories[0].labels, {'summer', 'sale', 'clearance'});
  });

  test('generated derive features work for abstract annotated classes', () {
    const left = EntityView('entity-1');
    const right = EntityView('entity-1');
    const different = EntityView('entity-2');

    expect(left.auditLabel(), 'audited');
    expect(left, equals(right));
    expect(left.hashCode, equals(right.hashCode));
    expect(left, isNot(equals(different)));
    expect(left.toString(), contains('Entity(id: entity-1)'));
  });

  test(
    'generated derive features include inherited fields on annotated subclasses',
    () {
      const left = DetailedEntity(
        'entity-1',
        label: 'Featured',
        tags: ['summer', 'sale'],
      );
      const right = DetailedEntity(
        'entity-1',
        label: 'Featured',
        tags: ['summer', 'sale'],
      );

      expect(left.auditLabel(), 'audited');
      expect(left, equals(right));
      expect(left.hashCode, equals(right.hashCode));
      expect(
        left.toString(),
        contains(
          'DetailedEntity(id: entity-1, label: Featured, tags: [summer, sale])',
        ),
      );
      expect(left.copyWith(), equals(left));

      final updated = left.copyWith(
        id: 'entity-2',
        tags: ['summer', 'clearance'],
      );
      expect(updated.id, 'entity-2');
      expect(updated.label, left.label);
      expect(updated.tags, ['summer', 'clearance']);
    },
  );

  test(
    'generated derive features work for classes with existing mixin chains',
    () {
      const left = TaggedValue(code: 'tag-1', aliases: ['featured', 'sale']);
      const right = TaggedValue(code: 'tag-1', aliases: ['featured', 'sale']);

      expect(left, equals(right));
      expect(left.hashCode, equals(right.hashCode));
      expect(left.copyWith(), equals(left));
      expect(left.auditLabel(), 'audited');
      expect(
        left.toString(),
        contains('TaggedValue(code: tag-1, aliases: [featured, sale])'),
      );

      final updated = left.copyWith(aliases: ['featured', 'clearance']);
      expect(updated.code, left.code);
      expect(updated.aliases, ['featured', 'clearance']);
    },
  );

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

    final replaced = original.copyWith(aliases: ['clearance']);
    expect(replaced.aliases, ['clearance']);
    expect(replaced.aliases, isNot(same(original.aliases)));

    final cleared = original.copyWith(note: null, aliases: null);
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

  test('generated serde features handle rename aliases and defaults', () {
    const profile = JsonProfile(
      id: 'user-1',
      displayName: 'May',
      tags: ['vip'],
    );

    expect(profile.toJson(), {
      'id': 'user-1',
      'display_name': 'May',
      'tags': ['vip'],
    });

    final fromAlias = JsonProfile.fromJson({
      'id': 'user-1',
      'displayName': 'May',
    });
    expect(
      fromAlias,
      equals(
        const JsonProfile(id: 'user-1', displayName: 'May', tags: ['guest']),
      ),
    );

    expect(
      () => JsonProfile.fromJson({
        'id': 'user-1',
        'unknown': true,
      }),
      throwsArgumentError,
    );
  });

  test('generated serde features support nested models and collections', () {
    const profile = JsonProfile(
      id: 'user-9',
      displayName: 'Aye',
      tags: ['featured'],
    );
    final account = JsonAccount(
      profile: profile,
      metrics: const {
        'yangon': [1, 2],
        'mandalay': [3],
      },
      archived: false,
    );

    expect(account.toJson(), {
      'profile': {
        'id': 'user-9',
        'display_name': 'Aye',
        'tags': ['featured'],
      },
      'metrics': {
        'yangon': [1, 2],
        'mandalay': [3],
      },
      'archived': false,
    });

    final roundTrip = JsonAccount.fromJson({
      'profile': {
        'id': 'user-9',
        'display_name': 'Aye',
        'tags': ['featured'],
      },
      'metrics': {
        'yangon': [1, 2],
        'mandalay': [3],
      },
      'archived': false,
    });

    expect(roundTrip, equals(account));
  });
}

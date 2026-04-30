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
    final price = Price(
      currency: 'USD',
      cents: 1999,
      tags: ['featured', 'sale'],
    );
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
      () => JsonProfile.fromJson({'id': 'user-1', 'unknown': true}),
      throwsArgumentError,
    );
  });

  test('generated serde diagnostics include the failing key for wrong types', () {
    expect(
      () => JsonProfile.fromJson({
        'id': 42,
        'display_name': 'May',
      }),
      throwsA(
        isA<ArgumentError>()
            .having((error) => error.name, 'name', 'id')
            .having(
              (error) => '${error.message}',
              'message',
              contains('expected String'),
            ),
      ),
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

  test('generated serde features support DateTime Uri and BigInt', () {
    final bundle = JsonScalarBundle(
      createdAt: DateTime.utc(2026, 4, 29, 12, 30, 45),
      updatedAt: null,
      website: Uri.parse('https://dust.dev/products/runner'),
      largeNumber: BigInt.parse('900719925474099312345'),
      endpoints: {
        Uri.parse('https://a.dust.dev'),
        Uri.parse('https://b.dust.dev'),
      },
      checkpoints: {
        'draft': DateTime.utc(2026, 4, 1, 8, 0, 0),
        'live': DateTime.utc(2026, 4, 29, 12, 30, 45),
      },
    );

    expect(bundle.toJson(), {
      'createdAt': '2026-04-29T12:30:45.000Z',
      'updatedAt': null,
      'website': 'https://dust.dev/products/runner',
      'largeNumber': '900719925474099312345',
      'endpoints': ['https://a.dust.dev', 'https://b.dust.dev'],
      'checkpoints': {
        'draft': '2026-04-01T08:00:00.000Z',
        'live': '2026-04-29T12:30:45.000Z',
      },
    });

    final roundTrip = JsonScalarBundle.fromJson({
      'createdAt': '2026-04-29T12:30:45.000Z',
      'updatedAt': '2026-05-01T09:15:00.000Z',
      'website': 'https://dust.dev/products/runner',
      'largeNumber': '900719925474099312345',
      'endpoints': ['https://a.dust.dev', 'https://b.dust.dev'],
      'checkpoints': {
        'draft': '2026-04-01T08:00:00.000Z',
        'live': '2026-04-29T12:30:45.000Z',
      },
    });

    expect(roundTrip.createdAt, DateTime.utc(2026, 4, 29, 12, 30, 45));
    expect(roundTrip.updatedAt, DateTime.utc(2026, 5, 1, 9, 15, 0));
    expect(roundTrip.website, Uri.parse('https://dust.dev/products/runner'));
    expect(roundTrip.largeNumber, BigInt.parse('900719925474099312345'));
    expect(roundTrip.endpoints, {
      Uri.parse('https://a.dust.dev'),
      Uri.parse('https://b.dust.dev'),
    });
    expect(roundTrip.checkpoints, {
      'draft': DateTime.utc(2026, 4, 1, 8, 0, 0),
      'live': DateTime.utc(2026, 4, 29, 12, 30, 45),
    });
  });

  test('generated serde diagnostics include the failing key for bad formats', () {
    expect(
      () => JsonScalarBundle.fromJson({
        'createdAt': 'not-a-date',
        'updatedAt': null,
        'website': 'https://dust.dev/products/runner',
        'largeNumber': '900719925474099312345',
        'endpoints': const <String>[],
        'checkpoints': const <String, String>{},
      }),
      throwsA(
        isA<ArgumentError>()
            .having((error) => error.name, 'name', 'createdAt')
            .having(
              (error) => '${error.message}',
              'message',
              contains('ISO-8601 DateTime string'),
            ),
      ),
    );
  });

  test('generated serde features support custom SerDeCodec fields', () {
    final bundle = JsonCodecBundle(
      createdAt: DateTime.fromMillisecondsSinceEpoch(
        1704067200000,
        isUtc: true,
      ),
      updatedAt: DateTime.fromMillisecondsSinceEpoch(
        1706745600000,
        isUtc: true,
      ),
    );

    expect(bundle.toJson(), {
      'createdAt': 1704067200000,
      'updatedAt': 1706745600000,
    });

    final roundTrip = JsonCodecBundle.fromJson({
      'createdAt': 1704067200000,
      'updatedAt': 1706745600000,
    });

    expect(roundTrip, equals(bundle));
  });

  test('generated serde features support enums and enum collections', () {
    const bundle = JsonEnumBundle(
      primaryLevel: AccessLevel.superAdmin,
      fallbackState: ReviewState.approved,
      levels: [
        AccessLevel.superAdmin,
        AccessLevel.readOnly,
      ],
      stateByRegion: {
        'yangon': ReviewState.pending,
        'mandalay': ReviewState.archived,
      },
      states: {
        ReviewState.pending,
        ReviewState.approved,
      },
    );

    expect(bundle.toJson(), {
      'primary_level': 'super-admin',
      'fallbackState': 'approved',
      'levels': ['super-admin', 'read-only'],
      'stateByRegion': {
        'yangon': 'pending',
        'mandalay': 'archived',
      },
      'states': ['pending', 'approved'],
    });

    final roundTrip = JsonEnumBundle.fromJson({
      'primaryLevel': 'guest-user',
      'fallbackState': null,
      'levels': ['guest-user', 'read-only'],
      'stateByRegion': {
        'yangon': 'approved',
        'mandalay': 'pending',
      },
      'states': ['approved', 'archived'],
    });

    expect(
      roundTrip,
      equals(
        const JsonEnumBundle(
          primaryLevel: AccessLevel.guestUser,
          fallbackState: null,
          levels: [
            AccessLevel.guestUser,
            AccessLevel.readOnly,
          ],
          stateByRegion: {
            'yangon': ReviewState.approved,
            'mandalay': ReviewState.pending,
          },
          states: {
            ReviewState.approved,
            ReviewState.archived,
          },
        ),
      ),
    );
  });

  test('generated serde diagnostics include unknown enum values', () {
    expect(
      () => JsonEnumBundle.fromJson({
        'primary_level': 'power-user',
        'fallbackState': 'approved',
        'levels': ['super-admin'],
        'stateByRegion': {'yangon': 'pending'},
        'states': ['approved'],
      }),
      throwsA(
        isA<ArgumentError>().having(
          (error) => '${error.message}',
          'message',
          contains('unknown value for AccessLevel'),
        ),
      ),
    );
  });

  test('generated serde features support enhanced enums with index codecs', () {
    const bundle = JsonEnhancedEnumBundle(
      primaryVehicle: Vehicle.car,
      fallbackVehicle: Vehicle.bicycle,
      fleet: [
        Vehicle.car,
        Vehicle.unicycle,
      ],
    );

    expect(bundle.toJson(), {
      'primaryVehicle': 0,
      'fallbackVehicle': 1,
      'fleet': [0, 2],
    });

    final roundTrip = JsonEnhancedEnumBundle.fromJson({
      'primaryVehicle': 2,
      'fallbackVehicle': null,
      'fleet': [1, 0],
    });

    expect(roundTrip.primaryVehicle, Vehicle.unicycle);
    expect(roundTrip.primaryVehicle.tires, 1);
    expect(roundTrip.primaryVehicle.isMotorized, isFalse);
    expect(roundTrip.fallbackVehicle, isNull);
    expect(roundTrip.fleet, [Vehicle.bicycle, Vehicle.car]);
    expect(roundTrip.fleet[0].tires, 2);
    expect(roundTrip.fleet[1].isMotorized, isTrue);
  });

  test('generated serde diagnostics include codec enum index failures', () {
    expect(
      () => JsonEnhancedEnumBundle.fromJson({
        'primaryVehicle': 99,
        'fallbackVehicle': null,
        'fleet': [0],
      }),
      throwsA(
        isA<ArgumentError>().having(
          (error) => '${error.message}',
          'message',
          contains('failed SerDeCodec decode'),
        ),
      ),
    );
  });

  test('generated serde diagnostics include the failing key for codec fields', () {
    expect(
      () => JsonCodecBundle.fromJson({
        'createdAt': null,
        'updatedAt': 1706745600000,
      }),
      throwsA(
        isA<ArgumentError>()
            .having((error) => error.name, 'name', 'createdAt')
            .having(
              (error) => '${error.message}',
              'message',
              contains('SerDeCodec'),
            ),
      ),
    );
  });

  test('generated serde features cover all supported SerDe flags', () {
    const options = JsonSerdeOptions(
      id: 'user-2',
      displayName: 'May',
      e: MyEnum.A,
      tags: ['vip'],
      serverOnly: 'server-secret',
      clientOnly: 'client-visible',
      hidden: 'hidden-secret',
    );

    expect(options.toJson(), {
      'id': 'user-2',
      'display_name': 'May',
      'e': 'A',
      'tags': ['vip'],
      'client_only': 'client-visible',
    });

    final fromJson = JsonSerdeOptions.fromJson({
      'id': 'user-2',
      'displayName': 'May',
      'e': 'B',
      'server_only': 'from-server',
      'client_only': 'ignored-client',
      'hidden': 'ignored-hidden',
    });

    expect(
      fromJson,
      equals(
        const JsonSerdeOptions(
          id: 'user-2',
          displayName: 'May',
          e: MyEnum.B,
          tags: ['guest'],
          serverOnly: 'from-server',
          clientOnly: 'client-default',
          hidden: 'hidden-default',
        ),
      ),
    );

    expect(
      () => JsonSerdeOptions.fromJson({
        'id': 'user-2',
        'display_name': 'May',
        'e': 'A',
        'unexpected': true,
      }),
      throwsArgumentError,
    );
  });
}

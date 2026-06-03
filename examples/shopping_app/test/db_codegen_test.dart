import 'dart:convert';

import 'package:dust_dart/db.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/data/cached_shopping_repository.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/core/db/shopping_cache_database.dart';
import 'package:shopping_app/features/products/models/product.dart';

void main() {
  test(
    'Dust DB generated shopping cache database maps rows and transactions',
    () async {
      final app = ShoppingCacheDatabase.open(':memory:');
      addTearDown(() async {
        await app.pool.close();
      });

      await app.pool.seedProductCache();

      expect(await app.pool.countCachedProducts(), 1);

      final product = await app.pool.findCachedProduct(7);
      expect(product, isNotNull);
      expect(product!.id, 7);
      expect(product.imageUrl, 'https://example.test/sneaker.png');
      expect(product.rating.rate, 4.8);
      expect(product.rating.count, 128);
      expect(product.pinned, isFalse);
      expect(product.payload.tags, <String>['featured', 'generated-db']);
      expect(product.payload.syncedBy, 'dust');
      expect(product.source, CacheSource.fakeStore);

      final products = await app.pool.listCachedProducts();
      expect(products.map((row) => row.title), <String>['Dust Runner']);

      const savedAt = '2026-05-26T10:30:00.000Z';
      await app.pool.transaction((tx) async {
        await tx.saveWishlist(7, 'Dust Runner', savedAt);
        return const Ok<void, SqlxError>(null);
      });

      final wishlist = await app.pool.listWishlist();
      expect(wishlist, hasLength(1));
      expect(wishlist.single.productId, 7);
      expect(wishlist.single.title, 'Dust Runner');
      expect(wishlist.single.savedAt, DateTime.parse(savedAt));
    },
  );

  test('cached repository uses generated DB in product load flow', () async {
    final app = ShoppingCacheDatabase.open(':memory:');
    addTearDown(() async {
      await app.pool.close();
    });

    final remote = _FlakyProductRepository();
    final repository = CachedShoppingRepository(remote: remote, database: app);

    final liveProducts = await repository.getProducts();
    expect(liveProducts.single.title, 'Dust Runner');
    expect(await app.pool.countCachedProducts(), 1);

    remote.failProducts = true;
    final cachedProducts = await repository.getProducts();
    expect(cachedProducts.single.title, 'Dust Runner');
    expect(
      cachedProducts.single.description,
      'Cached through the real product flow.',
    );
    expect(cachedProducts.single.rating.rate, 4.8);
  });
}

extension _ShoppingSeedQueries on SqlxDriver {
  Future<void> seedProductCache() async {
    await queryExecute(
      r'INSERT INTO product_cache (id, title, price, description, category, image, rating_rate, rating_count, payload, source) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)',
      [
        7,
        'Dust Runner',
        79.5,
        'Generated DB row used by the shopping flow.',
        'shoes',
        'https://example.test/sneaker.png',
        4.8,
        128,
        jsonEncode(<String, Object?>{
          'tags': <String>['featured', 'generated-db'],
          'syncedBy': 'dust',
        }),
        'fake_store',
      ],
    ).execute(this);
  }
}

final class _FlakyProductRepository implements ShoppingRepository {
  bool failProducts = false;

  static const _product = Product(
    id: 7,
    title: 'Dust Runner',
    price: 79.5,
    description: 'Cached through the real product flow.',
    category: 'shoes',
    image: 'https://example.test/sneaker.png',
    rating: Rating(rate: 4.8, count: 128),
  );

  @override
  Future<List<Product>> getProducts() async {
    if (failProducts) throw StateError('network down');
    return const [_product];
  }

  @override
  Future<Product> getProduct(int id) async {
    if (failProducts) throw StateError('network down');
    return _product;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

import 'dart:convert';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/data/cached_shopping_repository.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/core/db/shopping_cache_database.dart';
import 'package:shopping_app/features/products/models/product.dart';

void main() {
  test(
    'Database generated shopping cache maps rows and transactions',
    () async {
      final app = ShoppingCacheDatabase.open(':memory:');
      addTearDown(() async {
        await app.close();
      });

      expect(app.connection, isA<DatabaseConnection>());

      await app.connection.seedProductCache();
      expect(
        (app.pool as Sqlite3Executor)
            .database
            .select('SELECT 1')
            .single
            .columnAt(0),
        1,
      );

      expect(await app.connection.countCachedProducts(), 1);

      final product = await app.connection.findCachedProduct(7);
      expect(product, isNotNull);
      expect(product!.id, 7);
      expect(product.imageUrl, 'https://example.test/sneaker.png');
      expect(product.rating.rate, 4.8);
      expect(product.rating.count, 128);
      expect(product.pinned, isFalse);
      expect(product.payload.tags, <String>['featured', 'generated-db']);
      expect(product.payload.syncedBy, 'dust');
      expect(product.source, CacheSource.fakeStore);

      final products = await app.connection.listCachedProducts();
      expect(products.map((row) => row.title), <String>['Dust Runner']);

      const savedAt = '2026-05-26T10:30:00.000Z';
      await app.transaction((tx) async {
        await tx.saveWishlist(7, 'Dust Runner', savedAt);
        return const Ok<void, SqlxError>(null);
      });

      final wishlist = await app.connection.listWishlist();
      expect(wishlist, hasLength(1));
      expect(wishlist.single.productId, 7);
      expect(wishlist.single.title, 'Dust Runner');
      expect(wishlist.single.savedAt, DateTime.parse(savedAt));
    },
  );

  test('shopping cache applies reversible up migrations only', () async {
    final app = ShoppingCacheDatabase.open(':memory:');
    addTearDown(() async {
      await app.close();
    });

    final columns = await queryRaw(
      'PRAGMA table_info(product_cache)',
      const [],
    ).fetch(app.pool);
    expect(
      columns.map((row) => row.read<String>('name')),
      contains('last_synced_at'),
    );

    final migrations = await queryRaw(
      'SELECT name FROM __dust_schema_migrations ORDER BY name',
      const [],
    ).fetch(app.pool);
    expect(migrations.map((row) => row.read<String>('name')), <String>[
      '0001_shopping_cache.sql',
      '0002_product_cache_sync_metadata.up.sql',
    ]);
  });

  test('shopping cache accepts explicit SQLite connect options', () async {
    final app = ShoppingCacheDatabase.open(
      ':memory:',
      options: const SqliteConnectOptions.memory(
        foreignKeys: true,
        busyTimeout: Duration(milliseconds: 100),
      ),
    );
    addTearDown(() async {
      await app.close();
    });

    final database = (app.pool as Sqlite3Executor).database;
    expect(database.select('PRAGMA foreign_keys').single.columnAt(0), 1);
    expect(database.select('PRAGMA busy_timeout').single.columnAt(0), 100);

    await app.connection.seedProductCache();
    expect(await app.connection.countCachedProducts(), 1);
  });

  test('shopping cache generated DAO works inside transaction', () async {
    final app = ShoppingCacheDatabase.open(':memory:');
    addTearDown(() async {
      await app.close();
    });

    final result = await app.transaction<int>((tx) async {
      await tx.seedProductCache();
      return ShoppingCacheDao(tx).countCachedProducts();
    });

    expect(result.match(ok: (count) => count, err: (_) => -1), 1);
  });

  test('cached repository uses generated DB in product load flow', () async {
    final app = ShoppingCacheDatabase.open(':memory:');
    addTearDown(() async {
      await app.close();
    });

    final remote = _FlakyProductRepository();
    final repository = CachedShoppingRepository(remote: remote, database: app);

    final liveProducts = await repository.getProducts();
    expect(liveProducts.single.title, 'Dust Runner');
    expect(await app.connection.countCachedProducts(), 1);

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

extension _ShoppingSeedQueries on DatabaseExecutor {
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

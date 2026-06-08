import 'dart:convert';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

import '../../features/products/models/product.dart';
import 'shopping_cache_rows.dart';

export 'shopping_cache_rows.dart';

part 'shopping_cache_database.g.dart';

@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class ShoppingCacheDatabase {
  factory ShoppingCacheDatabase.open(String path) =
      _$ShoppingCacheDatabase.open;

  Pool get pool;
}

@SqlxDao()
abstract final class ShoppingCacheDao {
  const factory ShoppingCacheDao(Executor db) = _$ShoppingCacheDao;

  @Query(r'''
SELECT id, title, price, description, category, image,
       rating_rate, rating_count, payload, source
FROM product_cache
WHERE id = $1
''')
  Future<Result<CachedProductRow?, SqlxError>> findCachedProduct(int id);

  @Query(r'''
SELECT id, title, price, description, category, image,
       rating_rate, rating_count, payload, source
FROM product_cache
ORDER BY title
''')
  Future<Result<List<CachedProductRow>, SqlxError>> listCachedProducts();

  @Query(r'''SELECT COUNT(*) FROM product_cache''')
  Future<Result<int, SqlxError>> countCachedProducts();

  @Query(r'''
INSERT OR REPLACE INTO product_cache (
  id, title, price, description, category, image,
  rating_rate, rating_count, payload, source
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
''')
  Future<Result<ExecResult, SqlxError>> saveProductRow(
    int id,
    String title,
    double price,
    String description,
    String category,
    String image,
    double ratingRate,
    int ratingCount,
    String payload,
    String source,
  );

  @Query(r'''
INSERT OR REPLACE INTO wishlist_cache (product_id, title, saved_at)
VALUES ($1, $2, $3)
''')
  Future<Result<ExecResult, SqlxError>> saveWishlist(
    int productId,
    String title,
    String savedAt,
  );

  @Query(r'''
SELECT product_id, title, saved_at
FROM wishlist_cache
ORDER BY saved_at DESC
''')
  Future<Result<List<CachedWishlistRow>, SqlxError>> listWishlist();
}

extension ShoppingCacheQueries on Executor {
  Future<CachedProductRow?> findCachedProduct(int id) {
    return _unwrapSqlx(ShoppingCacheDao(this).findCachedProduct(id));
  }

  Future<List<CachedProductRow>> listCachedProducts() {
    return _unwrapSqlx(ShoppingCacheDao(this).listCachedProducts());
  }

  Future<int> countCachedProducts() {
    return _unwrapSqlx(ShoppingCacheDao(this).countCachedProducts());
  }

  Future<ExecResult> saveProduct(Product product) {
    return _unwrapSqlx(
      ShoppingCacheDao(this).saveProductRow(
        product.id,
        product.title,
        product.price,
        product.description,
        product.category,
        product.image,
        product.rating.rate,
        product.rating.count,
        jsonEncode(<String, Object?>{
          'tags': <String>[product.category, 'live-cache'],
          'syncedBy': 'shopping-flow',
        }),
        'fake_store',
      ),
    );
  }

  Future<ExecResult> saveWishlist(int productId, String title, String savedAt) {
    return _unwrapSqlx(
      ShoppingCacheDao(this).saveWishlist(productId, title, savedAt),
    );
  }

  Future<List<CachedWishlistRow>> listWishlist() {
    return _unwrapSqlx(ShoppingCacheDao(this).listWishlist());
  }
}

extension ShoppingProductCacheQueries on Executor {
  Future<void> replaceProductCache(List<Product> products) {
    return transaction((tx) async {
      for (final product in products) {
        await tx.saveProduct(product);
      }
      return const Ok<void, SqlxError>(null);
    }).then(
      (result) => result.match(
        ok: (_) {},
        err: (error) =>
            throw StateError('Failed to replace product cache: $error'),
      ),
    );
  }
}

Future<T> _unwrapSqlx<T>(Future<Result<T, SqlxError>> future) async {
  final result = await future;
  return result.match(
    ok: (value) => value,
    err: (error) => throw StateError('SQLx operation failed: $error'),
  );
}

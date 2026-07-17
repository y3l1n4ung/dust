import 'dart:convert';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

import '../../features/products/models/product.dart';
import 'shopping_cache_rows.dart';

export 'shopping_cache_rows.dart';

part 'shopping_cache_database.g.dart';

/// Shopping cache database model for the shopping app example.
@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class ShoppingCacheDatabase implements DatabaseClient {
  /// Creates a [ShoppingCacheDatabase] client.
  factory ShoppingCacheDatabase.open(String path) =
      _$ShoppingCacheDatabase.open;

  /// Open database connection.
  @override
  DatabaseConnection get connection;

  /// Backwards-compatible pool accessor.
  Pool get pool;
}

/// Shopping cache DAO.
@SqlxDao()
abstract final class ShoppingCacheDao {
  const factory ShoppingCacheDao(DatabaseExecutor db) = _$ShoppingCacheDao;

  /// Finds cached product.
  @Query(r'''
SELECT id, title, price, description, category, image,
       rating_rate, rating_count, payload, source
FROM product_cache
WHERE id = $1
''')
  Future<Result<CachedProductRow?, SqlxError>> findCachedProduct(int id);

  /// Lists cached products.
  @Query(r'''
SELECT id, title, price, description, category, image,
       rating_rate, rating_count, payload, source
FROM product_cache
ORDER BY title
''')
  Future<Result<List<CachedProductRow>, SqlxError>> listCachedProducts();

  /// Counts cached products.
  @Query(r'''SELECT COUNT(*) FROM product_cache''')
  Future<Result<int, SqlxError>> countCachedProducts();

  /// Saves product row.
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

  /// Saves wishlist.
  @Query(r'''
INSERT OR REPLACE INTO wishlist_cache (product_id, title, saved_at)
VALUES ($1, $2, $3)
''')
  Future<Result<ExecResult, SqlxError>> saveWishlist(
    int productId,
    String title,
    String savedAt,
  );

  /// Lists wishlist.
  @Query(r'''
SELECT product_id, title, saved_at
FROM wishlist_cache
ORDER BY saved_at DESC
''')
  Future<Result<List<CachedWishlistRow>, SqlxError>> listWishlist();
}

/// Shopping cache queries.
extension ShoppingCacheQueries on DatabaseExecutor {
  /// Finds cached product.
  Future<CachedProductRow?> findCachedProduct(int id) {
    return _unwrapSqlx(ShoppingCacheDao(this).findCachedProduct(id));
  }

  /// Lists cached products.
  Future<List<CachedProductRow>> listCachedProducts() {
    return _unwrapSqlx(ShoppingCacheDao(this).listCachedProducts());
  }

  /// Counts cached products.
  Future<int> countCachedProducts() {
    return _unwrapSqlx(ShoppingCacheDao(this).countCachedProducts());
  }

  /// Saves product.
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

  /// Saves wishlist.
  Future<ExecResult> saveWishlist(int productId, String title, String savedAt) {
    return _unwrapSqlx(
      ShoppingCacheDao(this).saveWishlist(productId, title, savedAt),
    );
  }

  /// Lists wishlist.
  Future<List<CachedWishlistRow>> listWishlist() {
    return _unwrapSqlx(ShoppingCacheDao(this).listWishlist());
  }
}

/// Shopping product cache queries.
extension ShoppingProductCacheQueries on DatabaseExecutor {
  /// Replaces product cache.
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

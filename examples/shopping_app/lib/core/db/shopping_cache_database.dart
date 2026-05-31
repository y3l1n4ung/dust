import 'dart:convert';

import 'package:dust_db_annotation/dust_db_annotation.dart';
import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

import '../../features/products/models/product.dart';

part 'shopping_cache_database.g.dart';

enum CacheSource { fakeStore, local }

final class CacheSourceFromString implements SqlxTryFrom<CacheSource, String> {
  const CacheSourceFromString();

  @override
  CacheSource decode(String value) => switch (value) {
    'fake_store' => CacheSource.fakeStore,
    'local' => CacheSource.local,
    _ => throw ArgumentError.value(value, 'value', 'Unknown cache source'),
  };
}

final class CachedProductPayload {
  const CachedProductPayload({required this.tags, required this.syncedBy});

  factory CachedProductPayload.fromJson(Map<String, Object?> json) {
    return CachedProductPayload(
      tags: (json['tags'] as List<Object?>).cast<String>(),
      syncedBy: json['syncedBy'] as String,
    );
  }

  final List<String> tags;
  final String syncedBy;
}

@Derive([FromRow()])
@Sqlx(renameAll: SqlxRename.snakeCase)
final class CachedProductRow {
  const CachedProductRow({
    required this.id,
    required this.title,
    required this.price,
    required this.description,
    required this.category,
    required this.imageUrl,
    required this.rating,
    this.pinned = false,
    required this.payload,
    required this.source,
  });

  final int id;
  final String title;
  final double price;
  final String description;
  final String category;

  @Sqlx(rename: 'image')
  final String imageUrl;

  @Sqlx(flatten: true)
  final CachedProductRatingRow rating;

  @Sqlx(skip: true, defaultValue: false)
  final bool pinned;

  @Sqlx(json: true)
  final CachedProductPayload payload;

  @Sqlx(tryFrom: CacheSourceFromString())
  final CacheSource source;

  Product toProduct() {
    return Product(
      id: id,
      title: title,
      price: price,
      description: description,
      category: category,
      image: imageUrl,
      rating: Rating(rate: rating.rate, count: rating.count),
    );
  }
}

@Derive([FromRow()])
final class CachedProductRatingRow {
  const CachedProductRatingRow({required this.rate, required this.count});

  @Sqlx(rename: 'rating_rate')
  final double rate;

  @Sqlx(rename: 'rating_count')
  final int count;
}

@Derive([FromRow()])
@Sqlx(renameAll: SqlxRename.snakeCase)
final class CachedWishlistRow {
  const CachedWishlistRow({
    required this.productId,
    required this.title,
    required this.savedAt,
  });

  final int productId;
  final String title;
  final DateTime savedAt;
}

@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class ShoppingCacheDatabase {
  factory ShoppingCacheDatabase.open(String path) =
      _$ShoppingCacheDatabase.open;

  Pool get pool;
}

@SqlxDao()
abstract final class ShoppingCacheDao {
  const factory ShoppingCacheDao(SqlxDriver db) = _$ShoppingCacheDao;

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

extension ShoppingCacheQueries on SqlxDriver {
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

  Future<ExecResult> saveWishlist(
    int productId,
    String title,
    String savedAt,
  ) {
    return _unwrapSqlx(
      ShoppingCacheDao(this).saveWishlist(productId, title, savedAt),
    );
  }

  Future<List<CachedWishlistRow>> listWishlist() {
    return _unwrapSqlx(ShoppingCacheDao(this).listWishlist());
  }
}

extension ShoppingProductCacheQueries on SqlxDriver {
  Future<void> replaceProductCache(List<Product> products) {
    return transaction((tx) async {
      for (final product in products) {
        await tx.saveProduct(product);
      }
      return const Ok<void, SqlxError>(null);
    }).then(
      (result) => result.match(
        ok: (_) {},
        err: (error) => throw StateError('Failed to replace product cache: $error'),
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

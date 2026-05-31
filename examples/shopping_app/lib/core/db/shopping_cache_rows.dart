import 'package:dust_db_annotation/dust_db_annotation.dart';
import 'package:dust_db_runtime/dust_db_runtime.dart';

import '../../features/products/models/product.dart';

part 'shopping_cache_rows.g.dart';

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

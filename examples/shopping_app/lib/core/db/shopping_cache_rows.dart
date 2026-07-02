import 'package:dust_dart/db.dart';

import '../../features/products/models/product.dart';

part 'shopping_cache_rows.g.dart';

/// Cache source values for the shopping app example.
enum CacheSource {
  /// Fake store cache source.
  fakeStore,

  /// Local cache source.
  local,
}

/// Cache source from string model for the shopping app example.
final class CacheSourceFromString implements SqlxTryFrom<CacheSource, String> {
  /// Creates a [CacheSourceFromString].
  const CacheSourceFromString();

  @override
  CacheSource decode(String value) => switch (value) {
        'fake_store' => CacheSource.fakeStore,
        'local' => CacheSource.local,
        _ => throw ArgumentError.value(value, 'value', 'Unknown cache source'),
      };
}

/// Cached product payload model for the shopping app example.
final class CachedProductPayload {
  /// Creates a [CachedProductPayload].
  const CachedProductPayload({required this.tags, required this.syncedBy});

  /// Creates a [CachedProductPayload] from JSON.
  factory CachedProductPayload.fromJson(Map<String, Object?> json) {
    return CachedProductPayload(
      tags: (json['tags'] as List<Object?>).cast<String>(),
      syncedBy: json['syncedBy'] as String,
    );
  }

  /// Tags.
  final List<String> tags;

  /// Synced by.
  final String syncedBy;
}

/// Cached product row model for the shopping app example.
@Derive([FromRow()])
@Sqlx(renameAll: SqlxRename.snakeCase)
final class CachedProductRow {
  /// Creates a [CachedProductRow].
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

  /// Unique identifier.
  final int id;

  /// Title.
  final String title;

  /// Price.
  final double price;

  /// Description.
  final String description;

  /// Category.
  final String category;

  /// Image URL.
  @Sqlx(rename: 'image')
  final String imageUrl;

  /// Rating.
  @Sqlx(flatten: true)
  final CachedProductRatingRow rating;

  /// Pinned.
  @Sqlx(skip: true, defaultValue: false)
  final bool pinned;

  /// Payload.
  @Sqlx(json: true)
  final CachedProductPayload payload;

  /// Source.
  @Sqlx(tryFrom: CacheSourceFromString())
  final CacheSource source;

  /// To product cache source.
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

/// Cached product rating row model for the shopping app example.
@Derive([FromRow()])
final class CachedProductRatingRow {
  /// Creates a [CachedProductRatingRow].
  const CachedProductRatingRow({required this.rate, required this.count});

  /// Rate.
  @Sqlx(rename: 'rating_rate')
  final double rate;

  /// Count.
  @Sqlx(rename: 'rating_count')
  final int count;
}

/// Cached wishlist row model for the shopping app example.
@Derive([FromRow()])
@Sqlx(renameAll: SqlxRename.snakeCase)
final class CachedWishlistRow {
  /// Creates a [CachedWishlistRow].
  const CachedWishlistRow({
    required this.productId,
    required this.title,
    required this.savedAt,
  });

  /// Product ID.
  final int productId;

  /// Title.
  final String title;

  /// Saved at.
  final DateTime savedAt;
}

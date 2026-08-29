import 'package:dust_dart/db.dart'
    show
        DatabaseExecutor,
        FromRow,
        QueryAs,
        Row,
        RowDeserializer,
        Sqlx,
        SqlxRename;
import 'package:dust_dart/serde.dart';

part 'latest_dart_derive_showcase.g.dart';

/// Latest product badge values for the product showcase example.
enum LatestProductBadge {
  /// Fresh latest product badge.
  fresh,

  /// Low stock latest product badge.
  lowStock,

  /// Sold out latest product badge.
  soldOut,
}

/// Latest dart product card.
@Derive([
  ToString(),
  Eq(),
  CopyWith(),
  Serialize(),
  Deserialize(),
  Validate(),
  FromRow(),
])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
@Sqlx(renameAll: SqlxRename.snakeCase)
final class LatestDartProductCard with _$LatestDartProductCard {
  /// Creates a [LatestDartProductCard].
  const LatestDartProductCard({
    required this.id,
    required this.title,
    required this.productUrl,
    required this.priceCents,
    required this.rating,
    required this.stockCount,
    required this.active,
    required this.launchedAt,
    this.internalOnly = false,
  });

  /// Creates a [LatestDartProductCard] from JSON.
  factory LatestDartProductCard.fromJson(Map<String, Object?> json) =>
      _$LatestDartProductCardFromJson(json);

  /// Unique identifier.
  @Validate(length: Length(min: 3), message: 'Product id is required')
  final String id;

  /// Title.
  @Validate(
    length: Length(min: 2, max: 80),
    message: 'Title must be 2-80 chars',
  )
  final String title;

  /// Product URL.
  @Validate(url: true, message: 'Product URL must be absolute')
  final String productUrl;

  /// Price cents.
  @Validate(range: Range(min: 1), message: 'Price must be positive')
  final int priceCents;

  /// Rating.
  @Validate(range: Range(min: 0, max: 5), message: 'Rating must be 0-5')
  final double rating;

  /// Stock count.
  @Validate(range: Range(min: 0), message: 'Stock cannot be negative')
  final int stockCount;

  /// Active.
  final bool active;

  /// Launched at.
  final DateTime launchedAt;

  /// Internal only.
  @SerDe(skip: true, defaultValue: false)
  @Sqlx(skip: true, defaultValue: false)
  final bool internalOnly;

  /// Product summary.
  ({String id, String title}) get summary => (id: id, title: title);

  /// Badge.
  LatestProductBadge get badge => switch ((active, stockCount)) {
        (false, _) => LatestProductBadge.soldOut,
        (true, <= 3) => LatestProductBadge.lowStock,
        _ => LatestProductBadge.fresh,
      };
}

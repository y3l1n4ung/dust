import 'package:dust_dart/db.dart'
    show FromRow, Row, Sqlx, SqlxRename, registerRowMapper;
import 'package:dust_dart/serde.dart';

part 'latest_dart_derive_showcase.g.dart';

enum LatestProductBadge { fresh, lowStock, soldOut }

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

  factory LatestDartProductCard.fromJson(Map<String, Object?> json) =>
      _$LatestDartProductCardFromJson(json);

  @Validate(length: Length(min: 3), message: 'Product id is required')
  final String id;

  @Validate(
    length: Length(min: 2, max: 80),
    message: 'Title must be 2-80 chars',
  )
  final String title;

  @Validate(url: true, message: 'Product URL must be absolute')
  final String productUrl;

  @Validate(range: Range(min: 1), message: 'Price must be positive')
  final int priceCents;

  @Validate(range: Range(min: 0, max: 5), message: 'Rating must be 0-5')
  final double rating;

  @Validate(range: Range(min: 0), message: 'Stock cannot be negative')
  final int stockCount;

  final bool active;
  final DateTime launchedAt;

  @SerDe(skip: true, defaultValue: false)
  @Sqlx(skip: true, defaultValue: false)
  final bool internalOnly;

  ({String id, String title}) get summary => (id: id, title: title);

  LatestProductBadge get badge => switch ((active, stockCount)) {
        (false, _) => LatestProductBadge.soldOut,
        (true, <= 3) => LatestProductBadge.lowStock,
        _ => LatestProductBadge.fresh,
      };
}

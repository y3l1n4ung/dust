import 'package:dust_dart/db.dart' show Row;
import 'package:dust_dart/derive.dart' show Invalid, ValidationError;
import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test(
    'latest Dart showcase exercises all public derive surfaces together',
    () {
      final launchedAt = DateTime.utc(2026, 6, 1, 10, 30);
      final card = LatestDartProductCard(
        id: 'sku-100',
        title: 'Runner',
        productUrl: 'https://dust.dev/products/runner',
        priceCents: 1299,
        rating: 4.5,
        stockCount: 2,
        active: true,
        launchedAt: launchedAt,
      );
      final sameCard = LatestDartProductCard(
        id: 'sku-100',
        title: 'Runner',
        productUrl: 'https://dust.dev/products/runner',
        priceCents: 1299,
        rating: 4.5,
        stockCount: 2,
        active: true,
        launchedAt: launchedAt,
      );

      expect(card, sameCard);
      expect(card.hashCode, sameCard.hashCode);
      expect(
        card.toString(),
        'LatestDartProductCard('
        'id: sku-100, '
        'title: Runner, '
        'productUrl: https://dust.dev/products/runner, '
        'priceCents: 1299, '
        'rating: 4.5, '
        'stockCount: 2, '
        'active: true, '
        'launchedAt: 2026-06-01 10:30:00.000Z, '
        'internalOnly: false'
        ')',
      );
      expect(card.summary, (id: 'sku-100', title: 'Runner'));
      expect(card.badge, LatestProductBadge.lowStock);
      expect(card.copyWith(stockCount: 7).badge, LatestProductBadge.fresh);
      expect(card.copyWith(active: false).badge, LatestProductBadge.soldOut);
    },
  );

  test('latest Dart showcase serializes and deserializes generated JSON', () {
    final launchedAt = DateTime.utc(2026, 6, 1, 10, 30);
    final card = LatestDartProductCard(
      id: 'sku-100',
      title: 'Runner',
      productUrl: 'https://dust.dev/products/runner',
      priceCents: 1299,
      rating: 4.5,
      stockCount: 2,
      active: true,
      launchedAt: launchedAt,
    );
    final json = <String, Object?>{
      'id': 'sku-100',
      'title': 'Runner',
      'product_url': 'https://dust.dev/products/runner',
      'price_cents': 1299,
      'rating': 4.5,
      'stock_count': 2,
      'active': true,
      'launched_at': '2026-06-01T10:30:00.000Z',
    };

    expect(card.toJson(), json);
    expect(LatestDartProductCard.fromJson(json), card);
  });

  test('latest Dart showcase validates model and form input', () {
    final invalid = LatestDartProductCard(
      id: 'x',
      title: 'R',
      productUrl: 'not-absolute',
      priceCents: 0,
      rating: 6,
      stockCount: -1,
      active: true,
      launchedAt: DateTime.utc(2026),
    );

    expect(
      invalid.validate(),
      const Invalid([
        ValidationError(field: 'id', message: 'Product id is required'),
        ValidationError(field: 'title', message: 'Title must be 2-80 chars'),
        ValidationError(
          field: 'productUrl',
          message: 'Product URL must be absolute',
        ),
        ValidationError(field: 'priceCents', message: 'Price must be positive'),
        ValidationError(field: 'rating', message: 'Rating must be 0-5'),
        ValidationError(
          field: 'stockCount',
          message: 'Stock cannot be negative',
        ),
      ]),
    );
    expect(validateLatestDartProductCardIdInput('x'), 'Product id is required');
    expect(
      validateLatestDartProductCardTitleInput('R'),
      'Title must be 2-80 chars',
    );
    expect(
      validateLatestDartProductCardProductUrlInput('not-absolute'),
      'Product URL must be absolute',
    );
    expect(
      validateLatestDartProductCardPriceCentsInput('0'),
      'Price must be positive',
    );
    expect(validateLatestDartProductCardRatingInput('6'), 'Rating must be 0-5');
    expect(
      validateLatestDartProductCardStockCountInput('-1'),
      'Stock cannot be negative',
    );
  });

  test('latest Dart showcase maps database rows through generated FromRow', () {
    final card = LatestDartProductCardFromRow.fromRow(
      _MapRow({
        'id': 'sku-100',
        'title': 'Runner',
        'product_url': 'https://dust.dev/products/runner',
        'price_cents': 1299,
        'rating': 4.5,
        'stock_count': 2,
        'active': 1,
        'launched_at': '2026-06-01T10:30:00.000Z',
      }),
    );

    expect(
      card,
      LatestDartProductCard(
        id: 'sku-100',
        title: 'Runner',
        productUrl: 'https://dust.dev/products/runner',
        priceCents: 1299,
        rating: 4.5,
        stockCount: 2,
        active: true,
        launchedAt: DateTime.utc(2026, 6, 1, 10, 30),
      ),
    );
  });
}

final class _MapRow implements Row {
  const _MapRow(this.values);

  final Map<String, Object?> values;

  @override
  T read<T>(String column) {
    final value = values[column];
    if (value is T) return value;
    throw StateError('Column $column is not $T');
  }

  @override
  T? readNullable<T>(String column) {
    final value = values[column];
    if (value == null) return null;
    if (value is T) return value as T;
    throw StateError('Column $column is not $T');
  }

  @override
  T readIndex<T>(int index) {
    throw UnsupportedError('index reads are not used by this test');
  }

  @override
  T? readIndexNullable<T>(int index) {
    throw UnsupportedError('index reads are not used by this test');
  }

  @override
  bool readBool(String column) {
    final value = values[column];
    return switch (value) {
      bool typed => typed,
      int typed when typed == 0 || typed == 1 => typed == 1,
      _ => throw StateError('Column $column is not bool-compatible'),
    };
  }

  @override
  bool? readBoolNullable(String column) {
    return values[column] == null ? null : readBool(column);
  }

  @override
  DateTime readDateTime(String column) {
    final value = values[column];
    return switch (value) {
      DateTime typed => typed.toUtc(),
      String typed => DateTime.parse(typed).toUtc(),
      _ => throw StateError('Column $column is not DateTime-compatible'),
    };
  }

  @override
  DateTime? readDateTimeNullable(String column) {
    return values[column] == null ? null : readDateTime(column);
  }
}

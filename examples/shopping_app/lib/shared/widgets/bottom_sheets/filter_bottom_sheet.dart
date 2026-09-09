import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../core/i18n/shop_i18n_keys.dart';

part 'filter_bottom_sheet_content.dart';

/// Filter options model for the shopping app example.
class FilterOptions {
  /// Category.
  final String? category;

  /// Min price.
  final double minPrice;

  /// Max price.
  final double maxPrice;

  /// Sort by.
  final SortOption sortBy;

  /// Creates a [FilterOptions].
  const FilterOptions({
    this.category,
    this.minPrice = 0,
    this.maxPrice = 1000,
    this.sortBy = SortOption.none,
  });

  /// Creates a copy with updated fields.
  FilterOptions copyWith({
    String? category,
    double? minPrice,
    double? maxPrice,
    SortOption? sortBy,
  }) {
    return FilterOptions(
      category: category ?? this.category,
      minPrice: minPrice ?? this.minPrice,
      maxPrice: maxPrice ?? this.maxPrice,
      sortBy: sortBy ?? this.sortBy,
    );
  }
}

/// Sort option values for the shopping app example.
enum SortOption {
  /// None sort option.
  none('None'),

  /// Price asc sort option.
  priceAsc('Price: Low to High'),

  /// Price desc sort option.
  priceDesc('Price: High to Low'),

  /// Rating sort option.
  rating('Rating'),

  /// Name sort option.
  name('Name');

  /// Label.
  final String label;
  const SortOption(this.label);

  /// Translation key.
  String get translationKey => switch (this) {
        SortOption.none => 'shop_sort_none',
        SortOption.priceAsc => 'shop_sort_price_low_full',
        SortOption.priceDesc => 'shop_sort_price_high_full',
        SortOption.rating => 'shop_sort_rating',
        SortOption.name => 'shop_sort_name',
      };
}

/// Filter bottom sheet.
class FilterBottomSheet {
  /// Show sort option.
  static Future<FilterOptions?> show({
    required BuildContext context,
    required List<String> categories,
    FilterOptions? currentFilters,
  }) {
    return showModalBottomSheet<FilterOptions>(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => _FilterBottomSheetContent(
        categories: categories,
        currentFilters: currentFilters ?? const FilterOptions(),
      ),
    );
  }
}

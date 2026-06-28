import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../core/i18n/shop_i18n_keys.dart';

class FilterOptions {
  final String? category;
  final double minPrice;
  final double maxPrice;
  final SortOption sortBy;

  const FilterOptions({
    this.category,
    this.minPrice = 0,
    this.maxPrice = 1000,
    this.sortBy = SortOption.none,
  });

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

enum SortOption {
  none('None'),
  priceAsc('Price: Low to High'),
  priceDesc('Price: High to Low'),
  rating('Rating'),
  name('Name');

  final String label;
  const SortOption(this.label);

  String get translationKey => switch (this) {
        SortOption.none => 'shop_sort_none',
        SortOption.priceAsc => 'shop_sort_price_low_full',
        SortOption.priceDesc => 'shop_sort_price_high_full',
        SortOption.rating => 'shop_sort_rating',
        SortOption.name => 'shop_sort_name',
      };
}

class FilterBottomSheet {
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

class _FilterBottomSheetContent extends StatefulWidget {
  final List<String> categories;
  final FilterOptions currentFilters;

  const _FilterBottomSheetContent({
    required this.categories,
    required this.currentFilters,
  });

  @override
  State<_FilterBottomSheetContent> createState() =>
      _FilterBottomSheetContentState();
}

class _FilterBottomSheetContentState extends State<_FilterBottomSheetContent> {
  late FilterOptions _filters;
  late RangeValues _priceRange;

  @override
  void initState() {
    super.initState();
    _filters = widget.currentFilters;
    _priceRange = RangeValues(
      widget.currentFilters.minPrice,
      widget.currentFilters.maxPrice,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).scaffoldBackgroundColor,
        borderRadius: const BorderRadius.vertical(top: Radius.circular(20)),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const SizedBox(height: 8),
          Container(
            width: 40,
            height: 4,
            decoration: BoxDecoration(
              color: Colors.grey[300],
              borderRadius: BorderRadius.circular(2),
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                TranslatedText(
                  'shop_filters',
                  defaultText: 'Filters',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                TextButton(
                  onPressed: () {
                    setState(() {
                      _filters = const FilterOptions();
                      _priceRange = const RangeValues(0, 1000);
                    });
                  },
                  child: const TranslatedText(
                    'shop_reset',
                    defaultText: 'Reset',
                  ),
                ),
              ],
            ),
          ),
          const Divider(height: 1),
          Flexible(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TranslatedText(
                    'shop_category',
                    defaultText: 'Category',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 8,
                    runSpacing: 8,
                    children: widget.categories.map((cat) {
                      final isSelected = _filters.category == cat;
                      return FilterChip(
                        label: TranslatedText.dynamic(
                          cat == 'all'
                              ? 'shop_category_all'
                              : shopCategoryKey(cat),
                          fallback: cat.toUpperCase(),
                        ),
                        selected: isSelected,
                        onSelected: (selected) {
                          setState(() {
                            _filters = _filters.copyWith(
                              category: selected ? cat : null,
                            );
                          });
                        },
                      );
                    }).toList(),
                  ),
                  const SizedBox(height: 24),
                  TranslatedText(
                    'shop_price_range',
                    defaultText: 'Price Range',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 12),
                  RangeSlider(
                    values: _priceRange,
                    min: 0,
                    max: 1000,
                    divisions: 100,
                    labels: RangeLabels(
                      context.tr(
                        'shop_product_price',
                        defaultText: r'${price}',
                        args: {'price': _priceRange.start.round()},
                      ),
                      context.tr(
                        'shop_product_price',
                        defaultText: r'${price}',
                        args: {'price': _priceRange.end.round()},
                      ),
                    ),
                    onChanged: (values) {
                      setState(() {
                        _priceRange = values;
                        _filters = _filters.copyWith(
                          minPrice: values.start,
                          maxPrice: values.end,
                        );
                      });
                    },
                  ),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      TranslatedText(
                        'shop_product_price',
                        defaultText: r'${price}',
                        args: {'price': _priceRange.start.round()},
                      ),
                      TranslatedText(
                        'shop_product_price',
                        defaultText: r'${price}',
                        args: {'price': _priceRange.end.round()},
                      ),
                    ],
                  ),
                  const SizedBox(height: 24),
                  TranslatedText(
                    'shop_sort_by',
                    defaultText: 'Sort By',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 12),
                  ...SortOption.values.map(
                    (option) => RadioListTile<SortOption>(
                      title: TranslatedText.dynamic(
                        option.translationKey,
                        fallback: option.label,
                      ),
                      value: option,
                      // ignore: deprecated_member_use
                      groupValue: _filters.sortBy,
                      // ignore: deprecated_member_use
                      onChanged: (v) => setState(
                        () => _filters = _filters.copyWith(sortBy: v),
                      ),
                      contentPadding: EdgeInsets.zero,
                    ),
                  ),
                ],
              ),
            ),
          ),
          Padding(
            padding: EdgeInsets.fromLTRB(
              16,
              16,
              16,
              16 + MediaQuery.of(context).padding.bottom,
            ),
            child: SizedBox(
              width: double.infinity,
              child: FilledButton(
                onPressed: () => Navigator.pop(context, _filters),
                child: const TranslatedText(
                  'shop_apply_filters',
                  defaultText: 'Apply Filters',
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

part of 'products_screen.dart';

// Filtering, sorting, and the product grid itself.

class _CategoryFilter extends StatelessWidget {
  final List<String> categories;
  final String selectedCategory;
  final ValueChanged<String> onCategorySelected;

  const _CategoryFilter({
    required this.categories,
    required this.selectedCategory,
    required this.onCategorySelected,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 50,
      child: ListView.builder(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        itemCount: categories.length,
        itemBuilder: (context, index) {
          final category = categories[index];
          final isSelected = category == selectedCategory;
          return Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4),
            child: FilterChip(
              label: TranslatedText.dynamic(
                category == 'all'
                    ? 'shop_category_all'
                    : shopCategoryKey(category),
                fallback: category.toUpperCase(),
              ),
              selected: isSelected,
              onSelected: (_) => onCategorySelected(category),
            ),
          );
        },
      ),
    );
  }
}

class _SearchAndSortBar extends StatelessWidget {
  const _SearchAndSortBar({
    required this.query,
    required this.sortOption,
    required this.onSearch,
    required this.onSort,
  });

  final String query;
  final ProductSortOption sortOption;
  final ValueChanged<String> onSearch;
  final ValueChanged<ProductSortOption> onSort;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
      child: Row(
        children: [
          Expanded(
            child: TextFormField(
              initialValue: query,
              decoration: InputDecoration(
                prefixIcon: const Icon(Icons.search),
                hintText: context.tr(
                  'shop_search_hint',
                  defaultText: 'Search products',
                ),
                border: const OutlineInputBorder(),
                isDense: true,
              ),
              onChanged: onSearch,
            ),
          ),
          const SizedBox(width: 12),
          DropdownButton<ProductSortOption>(
            value: sortOption,
            onChanged: (value) {
              if (value != null) onSort(value);
            },
            items: const [
              DropdownMenuItem(
                value: ProductSortOption.featured,
                child: TranslatedText(
                  'shop_sort_featured',
                  defaultText: 'Featured',
                ),
              ),
              DropdownMenuItem(
                value: ProductSortOption.priceLow,
                child: TranslatedText(
                  'shop_sort_price_low',
                  defaultText: 'Price ↑',
                ),
              ),
              DropdownMenuItem(
                value: ProductSortOption.priceHigh,
                child: TranslatedText(
                  'shop_sort_price_high',
                  defaultText: 'Price ↓',
                ),
              ),
              DropdownMenuItem(
                value: ProductSortOption.ratingHigh,
                child: TranslatedText(
                  'shop_sort_rating',
                  defaultText: 'Rating',
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _ProductsGrid extends StatelessWidget {
  final List<Product> products;

  const _ProductsGrid({required this.products});

  @override
  Widget build(BuildContext context) {
    if (products.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.search_off, size: 72, color: Colors.grey),
            SizedBox(height: 12),
            TranslatedText(
              'shop_no_products',
              defaultText: 'No products match your filters',
            ),
          ],
        ),
      );
    }

    return GridView.builder(
      padding: const EdgeInsets.all(16),
      gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: 220,
        childAspectRatio: 0.65,
        crossAxisSpacing: 16,
        mainAxisSpacing: 16,
      ),
      itemCount: products.length,
      itemBuilder: (context, index) {
        final product = products[index];
        return AnimatedGridItem(
          index: index,
          child: _ProductCard(product: product),
        );
      },
    );
  }
}

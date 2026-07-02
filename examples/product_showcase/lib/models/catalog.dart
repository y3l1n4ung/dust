import 'package:dust_dart/derive.dart';

import 'category.dart';
import 'product.dart';

part 'catalog.g.dart';

/// Inventory entry model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class InventoryEntry with _$InventoryEntry {
  /// Creates an [InventoryEntry].
  const InventoryEntry({
    required this.productSku,
    required this.warehouse,
    required this.quantity,
  });

  /// Product SKU.
  final String productSku;

  /// Warehouse.
  final String warehouse;

  /// Quantity.
  final int quantity;
}

/// Catalog model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class Catalog with _$Catalog {
  /// Creates a [Catalog].
  Catalog({
    required this.id,
    required this.products,
    required this.categoryById,
    required this.featuredSkus,
    required this.inventory,
  });

  /// Unique identifier.
  final String id;

  /// Products.
  final List<Product> products;

  /// Category by ID.
  final Map<String, Category> categoryById;

  /// Featured SKUs.
  final Set<String> featuredSkus;

  /// Inventory.
  final List<InventoryEntry> inventory;
}

void main(List<String> args) {
  final c = Catalog(
    id: "1",
    products: [],
    categoryById: {},
    featuredSkus: {},
    inventory: [],
  );
  print(c.hashCode);
  final d = c.copyWith(id: "3");

  print(d.hashCode);
}

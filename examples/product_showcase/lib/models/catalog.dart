import 'package:derive_annotation/derive_annotation.dart';

import 'category.dart';
import 'product.dart';

part 'catalog.g.dart';

@Derive([Debug(), Clone(), PartialEq(), Hash(), CopyWith()])
class InventoryEntry with _$InventoryEntryDust {
  const InventoryEntry({
    required this.productSku,
    required this.warehouse,
    required this.quantity,
  });

  final String productSku;
  final String warehouse;
  final int quantity;
}

@Derive([Debug(), Clone(), PartialEq(), Hash(), CopyWith()])
class Catalog with _$CatalogDust {
  const Catalog({
    required this.id,
    required this.products,
    required this.categoryById,
    required this.featuredSkus,
    required this.inventory,
  });

  final String id;
  final List<Product> products;
  final Map<String, Category> categoryById;
  final Set<String> featuredSkus;
  final List<InventoryEntry> inventory;
}

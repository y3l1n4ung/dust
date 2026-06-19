import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test('generated derive features work across multiple models', () {
    const price = Price(
      currency: 'USD',
      cents: 1999,
      tags: ['featured', 'sale'],
    );
    const samePrice = Price(
      currency: 'USD',
      cents: 1999,
      tags: ['featured', 'sale'],
    );
    expect(price, equals(samePrice));
    expect(price.hashCode, equals(samePrice.hashCode));
    expect(price.toString(), contains('Price('));

    const category = Category(
      id: 'cat-1',
      title: 'Shoes',
      labels: {'summer', 'sale'},
    );
    final updatedCategory = category.copyWith(title: 'Sneakers');
    expect(updatedCategory.title, 'Sneakers');
    expect(updatedCategory.labels, category.labels);

    final product = Product(
      sku: 'sku-1',
      title: 'Runner',
      price: price,
      categories: const [category],
      attributes: const {'color': 'black', 'size': '42'},
      featured: true,
    );
    final sameProduct = Product(
      sku: 'sku-1',
      title: 'Runner',
      price: samePrice,
      categories: const [
        Category(id: 'cat-1', title: 'Shoes', labels: {'sale', 'summer'}),
      ],
      attributes: const {'size': '42', 'color': 'black'},
      featured: true,
    );

    expect(product, equals(sameProduct));
    expect(product.hashCode, equals(sameProduct.hashCode));
    expect(product.copyWith(), equals(product));
    expect(product.copyWith(featured: false).featured, isFalse);

    final inventory = const [
      InventoryEntry(productSku: 'sku-1', warehouse: 'yangon-a', quantity: 10),
      InventoryEntry(productSku: 'sku-1', warehouse: 'yangon-b', quantity: 5),
    ];
    final catalog = Catalog(
      id: 'catalog-1',
      products: [product],
      categoryById: const {'cat-1': category},
      featuredSkus: const {'sku-1'},
      inventory: inventory,
    );
    final sameCatalog = Catalog(
      id: 'catalog-1',
      products: [sameProduct],
      categoryById: const {
        'cat-1': Category(
          id: 'cat-1',
          title: 'Shoes',
          labels: {'summer', 'sale'},
        ),
      },
      featuredSkus: const {'sku-1'},
      inventory: const [
        InventoryEntry(
          productSku: 'sku-1',
          warehouse: 'yangon-a',
          quantity: 10,
        ),
        InventoryEntry(productSku: 'sku-1', warehouse: 'yangon-b', quantity: 5),
      ],
    );

    expect(catalog, equals(sameCatalog));
    expect(catalog.hashCode, equals(sameCatalog.hashCode));
    expect(catalog.copyWith(id: 'catalog-2').id, 'catalog-2');
    expect(catalog.toString(), contains('Catalog('));
  });

  test('copyWith preserves nested model and collection references', () {
    final price = Price(
      currency: 'USD',
      cents: 1999,
      tags: ['featured', 'sale'],
    );
    final category = Category(
      id: 'cat-1',
      title: 'Shoes',
      labels: {'summer', 'sale'},
    );
    final product = Product(
      sku: 'sku-1',
      title: 'Runner',
      price: price,
      categories: [category],
      attributes: {'color': 'black'},
      featured: true,
    );

    final copied = product.copyWith();
    price.tags.add('vip');
    category.labels.add('clearance');

    expect(copied, isNot(same(product)));
    expect(copied.price, same(price));
    expect(copied.categories, same(product.categories));
    expect(copied.categories[0], same(category));
    expect(copied.price.tags, ['featured', 'sale', 'vip']);
    expect(copied.categories[0].labels, {'summer', 'sale', 'clearance'});

    final updated = product.copyWith(featured: false);
    price.tags.add('limited');
    category.labels.add('outlet');

    expect(updated.featured, isFalse);
    expect(updated.price, same(price));
    expect(updated.categories, same(product.categories));
    expect(updated.categories[0], same(category));
    expect(updated.price.tags, ['featured', 'sale', 'vip', 'limited']);
    expect(updated.categories[0].labels, {
      'summer',
      'sale',
      'clearance',
      'outlet',
    });
  });

  test('generated derive features work for abstract annotated classes', () {
    const left = EntityView('entity-1');
    const right = EntityView('entity-1');
    const different = EntityView('entity-2');

    expect(left.auditLabel(), 'audited');
    expect(left, equals(right));
    expect(left.hashCode, equals(right.hashCode));
    expect(left, isNot(equals(different)));
    expect(left.toString(), contains('Entity(id: entity-1)'));
  });

  test(
    'generated derive features include inherited fields on annotated subclasses',
    () {
      const left = DetailedEntity(
        'entity-1',
        label: 'Featured',
        tags: ['summer', 'sale'],
      );
      const right = DetailedEntity(
        'entity-1',
        label: 'Featured',
        tags: ['summer', 'sale'],
      );

      expect(left.auditLabel(), 'audited');
      expect(left, equals(right));
      expect(left.hashCode, equals(right.hashCode));
      expect(
        left.toString(),
        contains(
          'DetailedEntity(id: entity-1, label: Featured, tags: [summer, sale])',
        ),
      );
      expect(left.copyWith(), equals(left));

      final updated = left.copyWith(
        id: 'entity-2',
        tags: ['summer', 'clearance'],
      );
      expect(updated.id, 'entity-2');
      expect(updated.label, left.label);
      expect(updated.tags, ['summer', 'clearance']);
    },
  );

  test(
    'generated derive features work for classes with existing mixin chains',
    () {
      const left = TaggedValue(code: 'tag-1', aliases: ['featured', 'sale']);
      const right = TaggedValue(code: 'tag-1', aliases: ['featured', 'sale']);

      expect(left, equals(right));
      expect(left.hashCode, equals(right.hashCode));
      expect(left.copyWith(), equals(left));
      expect(left.auditLabel(), 'audited');
      expect(
        left.toString(),
        contains('TaggedValue(code: tag-1, aliases: [featured, sale])'),
      );

      final updated = left.copyWith(aliases: ['featured', 'clearance']);
      expect(updated.code, left.code);
      expect(updated.aliases, ['featured', 'clearance']);
    },
  );
}

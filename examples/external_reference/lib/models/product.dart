/// Product model
class Product {
  final String id;
  final String name;
  final String description;
  final double price;
  final String imageUrl;

  const Product({
    required this.id,
    required this.name,
    required this.description,
    required this.price,
    required this.imageUrl,
  });
}

/// Cart item with quantity
class CartItem {
  final Product product;
  final int quantity;

  const CartItem({
    required this.product,
    this.quantity = 1,
  });

  double get total => product.price * quantity;

  CartItem copyWith({int? quantity}) {
    return CartItem(
      product: product,
      quantity: quantity ?? this.quantity,
    );
  }
}

/// Sample products catalog
final sampleProducts = [
  const Product(
    id: '1',
    name: 'Wireless Headphones',
    description: 'Premium noise-canceling headphones',
    price: 299.99,
    imageUrl: 'headphones',
  ),
  const Product(
    id: '2',
    name: 'Smart Watch',
    description: 'Fitness tracking & notifications',
    price: 399.99,
    imageUrl: 'watch',
  ),
  const Product(
    id: '3',
    name: 'Laptop Stand',
    description: 'Ergonomic aluminum stand',
    price: 79.99,
    imageUrl: 'laptop',
  ),
  const Product(
    id: '4',
    name: 'Mechanical Keyboard',
    description: 'RGB backlit, cherry switches',
    price: 149.99,
    imageUrl: 'keyboard',
  ),
  const Product(
    id: '5',
    name: 'USB-C Hub',
    description: '7-in-1 multiport adapter',
    price: 59.99,
    imageUrl: 'usb',
  ),
  const Product(
    id: '6',
    name: 'Webcam HD',
    description: '1080p with auto-focus',
    price: 89.99,
    imageUrl: 'camera',
  ),
];

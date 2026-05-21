# Ease Shopping App

A complete e-commerce app demonstrating real-world Ease state management.

## Features

- User authentication with persistence
- Product catalog from FakeStore API
- Shopping cart with quantity management
- Checkout flow
- Profile management

## Running the App

```bash
cd apps/shopping_app
flutter pub get
dart run build_runner build
flutter run
```

## Architecture

### Feature-based Structure

```
lib/
├── core/
│   ├── logging/logger.dart       # App-wide logging
│   ├── services/api_service.dart # FakeStore API client
│   ├── services/storage_service.dart
│   └── router/app_router.dart
├── features/
│   ├── auth/
│   │   ├── models/user.dart, auth_state.dart
│   │   ├── view_models/auth_view_model.dart
│   │   └── views/login_screen.dart, register_screen.dart
│   ├── products/
│   │   ├── models/product.dart, products_state.dart
│   │   ├── view_models/products_view_model.dart
│   │   └── views/products_screen.dart
│   ├── cart/
│   │   ├── models/cart_item.dart, cart_state.dart
│   │   ├── view_models/cart_view_model.dart
│   │   └── views/cart_screen.dart
│   ├── checkout/
│   └── profile/
└── shared/
```

### State Management Patterns

**Persisted Auth State**
```dart
@Ease()
class AuthViewModel extends StateNotifier<AuthState> {
  AuthViewModel() : super(_loadInitialState());

  static AuthState _loadInitialState() {
    final token = StorageService.getString(StorageService.authTokenKey);
    if (token != null) {
      return AuthState(token: token, status: AuthStatus.authenticated);
    }
    return const AuthState(status: AuthStatus.unauthenticated);
  }
}
```

**Cart with Logging**
```dart
@Ease()
class CartViewModel extends StateNotifier<CartState> {
  void addToCart(Product product) {
    logger.userAction('add_to_cart', {'productId': product.id});
    state = state.copyWith(items: [...state.items, CartItem(product: product)]);
    logger.info('CART', 'Cart now has ${state.itemCount} items');
  }
}
```

**Computed Properties**
```dart
class CartState {
  final List<CartItem> items;

  int get itemCount => items.fold(0, (sum, item) => sum + item.quantity);
  double get totalPrice => items.fold(0.0, (sum, item) => sum + item.totalPrice);
  bool get isEmpty => items.isEmpty;
}
```

## API Integration

Uses [FakeStore API](https://fakestoreapi.com/) for:
- Product listing
- User authentication
- User registration

## License

MIT

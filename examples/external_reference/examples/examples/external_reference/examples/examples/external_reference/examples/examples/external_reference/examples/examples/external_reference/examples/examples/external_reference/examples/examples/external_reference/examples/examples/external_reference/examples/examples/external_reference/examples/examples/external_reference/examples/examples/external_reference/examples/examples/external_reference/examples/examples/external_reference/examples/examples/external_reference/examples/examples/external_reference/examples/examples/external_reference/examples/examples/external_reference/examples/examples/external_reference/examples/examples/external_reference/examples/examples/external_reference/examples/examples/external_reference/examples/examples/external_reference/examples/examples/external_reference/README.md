# Ease Example App

Comprehensive examples demonstrating Ease state management patterns.

## Running the App

```bash
cd apps/example
flutter pub get
dart run build_runner build
flutter run
```

## Examples

| Example | Description |
|---------|-------------|
| Counter | Basic increment/decrement with `StateNotifier<int>` |
| Todo | CRUD operations with list state |
| Cart | Shopping cart with computed values (subtotal, tax, total) |
| Auth | Login/logout with SharedPreferences persistence |
| Form | Form validation with real-time feedback |
| Theme | Light/dark mode switching |
| Search | Debounced search with async operations |
| Pagination | Infinite scroll with load more |
| Network | Real API calls with loading/error states |

## Key Patterns

### Basic Usage (Counter)

```dart
// view_models/counter_view_model.dart
@Ease()
class CounterViewModel extends StateNotifier<int> {
  CounterViewModel() : super(0);

  void increment() => state++;
  void decrement() => state--;
  void reset() => state = 0;
}

// views/counter_view.dart
final counter = context.counterViewModel;
Text('${counter.state}');
FloatingActionButton(onPressed: counter.increment);
```

### Complex State with copyWith (Cart)

```dart
class CartStatus {
  final List<CartItem> items;
  final bool isLoading;

  int get itemCount => items.fold(0, (sum, item) => sum + item.quantity);
  double get subtotal => items.fold(0, (sum, item) => sum + item.total);
  double get tax => subtotal * 0.1;
  double get total => subtotal + tax;

  CartStatus copyWith({List<CartItem>? items, bool? isLoading}) { ... }
}

@Ease()
class CartViewModel extends StateNotifier<CartStatus> {
  void addToCart(Product product) {
    state = state.copyWith(items: [...state.items, CartItem(product: product)]);
  }
}
```

### Selector Pattern (Granular Rebuilds)

```dart
// Only rebuilds when itemCount changes, not when isLoading changes
class CartBadge extends StatelessWidget {
  Widget build(BuildContext context) {
    final count = context.selectCartViewModel((s) => s.itemCount);
    return Text('$count');
  }
}
```

### Persistence (Auth)

```dart
@Ease()
class AuthViewModel extends StateNotifier<AuthStatus> {
  Future<void> login(String email, String password) async {
    state = state.copyWith(isLoading: true);
    // ... API call ...
    await _saveUser(user);  // SharedPreferences
    state = state.copyWith(user: user, isLoading: false);
  }
}
```

## License

MIT

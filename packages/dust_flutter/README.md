# dust_flutter

Flutter runtime and annotations for Dust generated routing and state code.

You focus on product. We focus on performance.

## Our Promise

- Routing and state management APIs are 50% stable and may still be refined
  before stabilization.
- Generated code can improve while app widgets and product logic stay focused.
- No external routing or state-management package is required by Dust.

## Public surfaces

- `package:dust_flutter/route.dart`: Navigator 2.0 annotations and runtime.
- `package:dust_flutter/state.dart`: ViewModel annotations and runtime.
- `package:dust_flutter/dust_flutter.dart`: convenience export for all
  Flutter-only APIs.

## Routing

```dart
import 'package:dust_flutter/route.dart';

@Router(initial: '/', notFound: '/404')
final class AppRouter extends $AppRouter {
  AppRouter({required this.auth});

  final AuthViewModel auth;

  @override
  AppRoutePath? redirect(AppRoutePath route) {
    if (!auth.isLoggedIn && route.requiresAuth) {
      return LoginRoute(from: route.location);
    }
    return null;
  }
}
```

## State

```dart
import 'package:dust_flutter/state.dart';

@ViewModel(state: CounterState)
final class CounterViewModel extends $CounterViewModel {
  CounterViewModel(super.args);

  void increment() {
    emit(state.copyWith(count: state.count + 1));
  }
}
```

See the canonical guides in `docs/usage/routing.md` and
`docs/usage/state.md`.

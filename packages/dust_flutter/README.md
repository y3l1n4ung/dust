# dust_flutter

Flutter runtime and annotations for Dust generated routing, state, and i18n code.

You focus on product. We focus on performance.

## Our Promise

- Routing, state management, and i18n APIs are beta and may still be refined
  before stabilization.
- Generated code can improve while app widgets and product logic stay focused.
- No external routing or state-management package is required by Dust.

## Public surfaces

- `package:dust_flutter/route.dart`: Navigator 2.0 annotations and runtime.
- `package:dust_flutter/state.dart`: ViewModel annotations and runtime.
- `package:dust_flutter/i18n.dart`: i18n runtime scope, controller, and widgets.
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

## i18n

```dart
import 'package:dust_flutter/i18n.dart';

final i18n = I18nController(
  config: const I18nConfig(
    locales: ['en', 'my'],
    fallbackLocale: 'en',
  ),
);

await i18n.loadAssetBundles();

I18nScope(
  controller: i18n,
  child: const TranslatedText('home_title'),
);
```

Runtime keys use a namespace prefix followed by an underscore. For example,
`home_title` loads from `assets/i18n/{locale}/home.arb` and reads the ARB
message key `title`. `home_title_name` reads the `title_name` key from the same
file.

See the canonical guides in `docs/usage/routing.md`, `docs/usage/state.md`,
and `docs/usage/i18n.md`.

# dust_flutter

Flutter annotations and runtime APIs for Dust-generated routing, state
management, and i18n.

## Status

- `0.1.0` is the first public Flutter runtime release.
- `0.1.1` adds opt-in `AppRouter` debug diagnostics for generated routing.
- `0.1.2` makes generated route pushes awaitable and applies route-specific
  transitions at the page route boundary.
- Routing, state management, and i18n APIs are beta and may still receive
  compatibility-preserving refinements before stabilization.
- Generated code can improve while app widgets and product logic stay focused.
- No external routing or state-management package is required by Dust.

## Install

Add `dust_flutter` to Flutter apps that use Dust-generated Flutter code:

```yaml
dependencies:
  dust_flutter: ^0.1.2
```

Most projects also use `dust_dart` for model derives and serialization:

```yaml
dependencies:
  dust_dart: ^0.1.0
  dust_flutter: ^0.1.2
```

Run Dust from your package root after adding annotations:

```sh
dust build
```

## Imports

- `package:dust_flutter/route.dart`: Navigator 2.0 annotations and runtime.
- `package:dust_flutter/state.dart`: ViewModel annotations and runtime.
- `package:dust_flutter/i18n.dart`: i18n runtime scope, controller, and widgets.
- `package:dust_flutter/dust_flutter.dart`: convenience export for all
  Flutter-only APIs.

## How to annotate

Use normal Flutter widgets and ViewModel classes, then add Dust annotations:

```dart
import 'package:dust_flutter/route.dart';
import 'package:dust_flutter/state.dart';
import 'package:flutter/widgets.dart';

part 'app.g.dart';

class CounterState {
  const CounterState({this.count = 0});

  final int count;

  CounterState copyWith({int? count}) {
    return CounterState(count: count ?? this.count);
  }
}

@AppRoute('/products/:id', name: 'product')
final class ProductPage extends StatelessWidget {
  const ProductPage({required this.id, this.tab, super.key});

  final int id;
  final String? tab;

  @override
  Widget build(BuildContext context) => Text('Product $id ${tab ?? ''}');
}

@ViewModel(state: CounterState)
final class CounterViewModel extends $CounterViewModel {
  CounterViewModel(super.args);

  void increment() {
    emit(state.copyWith(count: state.count + 1));
  }
}
```

Run generation from the app package root:

```sh
dust build
```

Full guides:

- [State management annotations](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/state.md)
- [Typed routing annotations](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/routing.md)
- [i18n runtime setup](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/i18n.md)
- [Package example](https://github.com/y3l1n4ung/dust/blob/main/packages/dust_flutter/example/dust_flutter_example.dart)

## Routing

```dart
import 'package:dust_flutter/route.dart';

@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
  RootRouter({required this.auth});

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

Override `debugLogDiagnostics` to print `AppRouter:` route-table, redirect,
guard, and stack diagnostics while debugging.

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

## Documentation

See the canonical Dust usage docs at
[github.com/y3l1n4ung/dust/tree/main/docs/usage](https://github.com/y3l1n4ung/dust/tree/main/docs/usage).

# Typed Routing

Dust Routing generates a type-safe Flutter Navigator 2.0 router from annotations. It creates a typed route tree that provides compile-time safety for parameters and navigation.

---

## Installation

Add the router runtime package to your `pubspec.yaml`:

```yaml
dependencies:
  dust_flutter: ^0.1.0
```

---

## Quick Start

### 1. Define the Router
Create an entrypoint (usually `lib/route.dart`) and annotate it with `@Router`.

```dart
import 'package:flutter/material.dart' hide Route, Router;
import 'package:dust_flutter/route.dart';
import 'route.g.dart';

export 'route.g.dart';

@Router(
  initial: HomePage,
  notFound: NotFoundPage,
)
final class AppRouter extends $AppRouter {}
```

### 2. Annotate Pages
Annotate your widgets with `@Route`.

```dart
@Route('/', name: 'home')
class HomePage extends StatelessWidget { ... }

@Route('/profile/:userId', name: 'profile')
class ProfilePage extends StatelessWidget {
  const ProfilePage({required this.userId});
  final String userId;
}
```

### 3. Initialize MaterialApp
```dart
MaterialApp.router(
  routerConfig: AppRouter().config,
);
```

---

## Configuration Reference

### `@Router` Options (Class Level)

| Property | Type | Description |
| :--- | :--- | :--- |
| `initial` | `Type` | **Required.** The widget class shown when the app starts without a deep link. |
| `notFound` | `Type` | The widget class shown when a URL doesn't match any route. |
| `refreshListenable` | `String` | Name of a field (e.g., `authService`) to watch. Triggers redirects when it changes. |
| `generatedBase` | `String` | Custom name for the generated base class. Defaults to `$ClassName`. |

### `@Route` Options (Widget Level)

| Property | Type | Description |
| :--- | :--- | :--- |
| `path` | `String` | **Required.** The URL template (e.g., `/users/:id`). |
| `name` | `String` | Unique name for navigation. If omitted, derived from the class name. |
| `shell` | `Type` | A widget class (like `MainLayout`) that wraps this page. |
| `guards` | `List<Type>` | List of `RouteGuard` classes to run before entering. |
| `transition` | `PageTransitionsBuilder` | Custom transition for this specific route. |
| `fullscreenDialog` | `bool` | If `true`, the page is presented as a modal fullscreen dialog. |
| `maintainState` | `bool` | Whether to keep the page state alive when inactive. Defaults to `true`. |

---

## Navigation API

Dust generates type-safe extension methods on `BuildContext`.

```dart
// Navigate to home
context.routes.home().go();

// Navigate with typed parameters
context.routes.profile(userId: '42').push();

// Replace current route
context.routes.home().replace();
```

> [!TIP]
> **Route Naming:** If you omit the `name` property, Dust automatically creates one by lower-camel-casing the class name and removing suffixes like `Page`, `Screen`, or `View`. (e.g., `UserProfilePage` -> `userProfile`).

---

## Guards and Redirects

Use `RouteGuard` to protect specific routes based on app state (e.g., authentication).

```dart
final class AuthGuard implements RouteGuard<AppRoutePath> {
  @override
  Future<RouteGuardResult<AppRoutePath>> canActivate(RouteState state) async {
    if (!isAuthenticated) return RouteGuardResult.redirect(LoginRoute());
    return RouteGuardResult.allow();
  }
}

@Route('/profile', guards: [AuthGuard])
class ProfilePage extends StatelessWidget { ... }
```

---

## Deep Linking

Dust handles deep links and browser URLs automatically.

| URL | Resolved Route |
| :--- | :--- |
| `/` | `HomeRoute()` |
| `/profile/abc` | `ProfileRoute(userId: 'abc')` |
| `/profile/42?tab=settings` | `ProfileRoute(userId: '42', tab: 'settings')` |

> [!IMPORTANT]
> **Web Browser Refresh:**
> Dust preserves the navigation stack during browser refreshes by serializing the stack state into the browser history API.

---

## Migration Guide

**Coming from `go_router` or `auto_route`?**

| Feature | `go_router` | `auto_route` | Dust |
| :--- | :--- | :--- | :--- |
| Config | `GoRouter(...)` | `@AutoRouter(...)` | `@Router(...)` |
| Route Def | `GoRoute(...)` | `@AdaptiveRoute(...)` | `@Route(...)` |
| Parameters | String-based | Typed | **Typed** |
| Build Tool | `build_runner` | `build_runner` | **Standalone Binary** |

---

## Generation Output

Dust generates a typed `AppRoutePath` sealed class and a `RouterDelegate`. Below is a preview of the generated structure:

```dart
// route.g.dart (Simplified)
sealed class AppRoutePath {
  String get location;
}

final class ProfileRoute extends AppRoutePath {
  final String userId;
  ProfileRoute({required this.userId});
  
  @override
  String get location => '/profile/$userId';
}
```

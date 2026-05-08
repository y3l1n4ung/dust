import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../views/cart_view.dart';
import '../views/chat_view.dart';
import '../views/counter_view.dart';
import '../views/drawing_view.dart';
import '../views/form_view.dart';
import '../views/login_view.dart';
import '../views/network_view.dart';
import '../views/pagination_view.dart';
import '../views/profile_view.dart';
import '../views/search_view.dart';
import '../views/side_effect_view.dart';
import '../views/theme_view.dart';
import '../views/todo_view.dart';
import '../view_models/auth_view_model.dart';

/// App routes
class AppRoutes {
  static const home = '/';
  static const login = '/login';
  static const counter = '/counter';
  static const todo = '/todo';
  static const form = '/form';
  static const theme = '/theme';
  static const profile = '/profile';
  static const cart = '/cart';
  static const search = '/search';
  static const pagination = '/pagination';
  static const network = '/network';
  static const drawing = '/drawing';
  static const sideEffects = '/side-effects';
  static const chat = '/chat';
}

/// Creates the app router with auth viewmodel for navigation guards
GoRouter createRouter(AuthViewModel authViewModel) {
  return GoRouter(
    initialLocation: AppRoutes.home,
    debugLogDiagnostics: true,

    // Refresh when auth state changes
    refreshListenable: authViewModel,

    // Global redirect for auth guards
    redirect: (context, state) {
      final isAuthenticated = authViewModel.state.isAuthenticated;
      final currentLocation = state.matchedLocation;
      final isLoggingIn = currentLocation == AppRoutes.login;

      // Protected routes that require authentication
      final protectedRoutes = [AppRoutes.profile];
      final isProtectedRoute = protectedRoutes.contains(currentLocation);

      // Redirect to home if on protected route and logged out
      if (isProtectedRoute && !isAuthenticated) {
        return AppRoutes.home;
      }

      // Redirect away from login if already authenticated
      if (isLoggingIn && isAuthenticated) {
        final redirect = state.uri.queryParameters['redirect'];
        return redirect ?? AppRoutes.home;
      }

      return null; // No redirect
    },

    routes: [
      GoRoute(
        path: AppRoutes.home,
        name: 'home',
        builder: (context, state) => const HomeView(),
      ),
      GoRoute(
        path: AppRoutes.login,
        name: 'login',
        builder: (context, state) => const LoginView(),
      ),
      GoRoute(
        path: AppRoutes.counter,
        name: 'counter',
        builder: (context, state) => const CounterView(),
      ),
      GoRoute(
        path: AppRoutes.todo,
        name: 'todo',
        builder: (context, state) => const TodoView(),
      ),
      GoRoute(
        path: AppRoutes.form,
        name: 'form',
        builder: (context, state) => const FormView(),
      ),
      GoRoute(
        path: AppRoutes.theme,
        name: 'theme',
        builder: (context, state) => const ThemeView(),
      ),
      GoRoute(
        path: AppRoutes.profile,
        name: 'profile',
        builder: (context, state) => const ProfileView(),
      ),
      GoRoute(
        path: AppRoutes.cart,
        name: 'cart',
        builder: (context, state) => const CartView(),
      ),
      GoRoute(
        path: AppRoutes.search,
        name: 'search',
        builder: (context, state) => const SearchView(),
      ),
      GoRoute(
        path: AppRoutes.pagination,
        name: 'pagination',
        builder: (context, state) => const PaginationView(),
      ),
      GoRoute(
        path: AppRoutes.network,
        name: 'network',
        builder: (context, state) => const NetworkView(),
      ),
      GoRoute(
        path: AppRoutes.drawing,
        name: 'drawing',
        builder: (context, state) => const DrawingView(),
      ),
      GoRoute(
        path: AppRoutes.sideEffects,
        name: 'side-effects',
        builder: (context, state) => const SideEffectView(),
      ),
      GoRoute(
        path: AppRoutes.chat,
        name: 'chat',
        builder: (context, state) => const ChatView(),
      ),
    ],

    errorBuilder: (context, state) => Scaffold(
      appBar: AppBar(title: const Text('Error')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 64, color: Colors.red),
            const SizedBox(height: 16),
            Text('Page not found: ${state.matchedLocation}'),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () => context.go(AppRoutes.home),
              child: const Text('Go Home'),
            ),
          ],
        ),
      ),
    ),
  );
}

/// Home View with navigation
class HomeView extends StatelessWidget {
  const HomeView({super.key});

  @override
  Widget build(BuildContext context) {
    final authViewModel = context.authViewModel;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Ease Examples'),
        centerTitle: true,
        actions: [
          // Auth status indicator
          if (authViewModel.state.isAuthenticated)
            IconButton(
              icon: const Icon(Icons.person),
              onPressed: () => context.push(AppRoutes.profile),
              tooltip: authViewModel.state.user?.name ?? 'Profile',
            )
          else
            TextButton.icon(
              icon: const Icon(Icons.login),
              label: const Text('Login'),
              onPressed: () => context.go(AppRoutes.login),
            ),
          // Theme toggle
          IconButton(
            icon: const Icon(Icons.palette),
            onPressed: () => context.push(AppRoutes.theme),
            tooltip: 'Theme',
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // Auth status card
          if (authViewModel.state.isAuthenticated)
            Card(
              color: Theme.of(context).colorScheme.primaryContainer,
              child: ListTile(
                leading: const CircleAvatar(child: Icon(Icons.person)),
                title: Text('Welcome, ${authViewModel.state.user?.name}!'),
                subtitle: Text(authViewModel.state.user?.email ?? ''),
                trailing: TextButton(
                  onPressed: authViewModel.logout,
                  child: const Text('Logout'),
                ),
              ),
            ),
          if (authViewModel.state.isAuthenticated) const SizedBox(height: 16),

          // Section: Basic Examples
          _SectionHeader(title: 'Basic Examples'),
          const SizedBox(height: 8),
          _NavCard(
            title: 'Counter',
            subtitle: 'Simple state with int',
            icon: Icons.add_circle_outline,
            color: Colors.blue,
            route: AppRoutes.counter,
          ),
          const SizedBox(height: 12),
          _NavCard(
            title: 'Todo List',
            subtitle: 'List state management',
            icon: Icons.checklist,
            color: Colors.green,
            route: AppRoutes.todo,
          ),
          const SizedBox(height: 12),
          _NavCard(
            title: 'Registration Form',
            subtitle: 'Complex form with validation',
            icon: Icons.app_registration,
            color: Colors.orange,
            route: AppRoutes.form,
          ),
          const SizedBox(height: 24),

          // Section: Enterprise Examples
          _SectionHeader(title: 'Enterprise Examples'),
          const SizedBox(height: 8),
          _NavCard(
            title: 'Shopping Cart',
            subtitle: 'Cart with computed totals',
            icon: Icons.shopping_cart,
            color: Colors.purple,
            route: AppRoutes.cart,
          ),
          const SizedBox(height: 12),
          _NavCard(
            title: 'Search',
            subtitle: 'Debounced search with results',
            icon: Icons.search,
            color: Colors.teal,
            route: AppRoutes.search,
          ),
          const SizedBox(height: 12),
          _NavCard(
            title: 'Infinite Scroll',
            subtitle: 'Pagination with load more',
            icon: Icons.view_list,
            color: Colors.indigo,
            route: AppRoutes.pagination,
          ),
          const SizedBox(height: 12),
          _NavCard(
            title: 'API Fetch',
            subtitle: 'Real network calls with caching',
            icon: Icons.cloud_download,
            color: Colors.cyan,
            route: AppRoutes.network,
          ),
          const SizedBox(height: 24),

          // Section: Advanced Features
          _SectionHeader(title: 'Advanced Features'),
          const SizedBox(height: 8),
          _NavCard(
            title: 'Drawing (Undo/Redo)',
            subtitle: 'Time machine middleware demo',
            icon: Icons.undo,
            color: Colors.deepOrange,
            route: AppRoutes.drawing,
          ),
          const SizedBox(height: 12),
          _NavCard(
            title: 'Side Effects & Streaming',
            subtitle: 'Async, debounce, streams, timers',
            icon: Icons.bolt,
            color: Colors.amber,
            route: AppRoutes.sideEffects,
          ),
          const SizedBox(height: 12),
          _NavCard(
            title: 'Chat Example',
            subtitle: 'WebSocket-like streaming',
            icon: Icons.chat,
            color: Colors.pink,
            route: AppRoutes.chat,
          ),
          const SizedBox(height: 24),

          // Section: Auth & Protected
          _SectionHeader(title: 'Auth & Protected'),
          const SizedBox(height: 8),
          _NavCard(
            title: 'Profile (Protected)',
            subtitle: 'Requires authentication',
            icon: Icons.lock,
            color: Colors.red,
            route: AppRoutes.profile,
          ),
        ],
      ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  final String title;

  const _SectionHeader({required this.title});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(left: 4),
      child: Text(
        title,
        style: Theme.of(context).textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.bold,
              color: Theme.of(context).colorScheme.primary,
            ),
      ),
    );
  }
}

class _NavCard extends StatelessWidget {
  final String title;
  final String subtitle;
  final IconData icon;
  final Color color;
  final String route;

  const _NavCard({
    required this.title,
    required this.subtitle,
    required this.icon,
    required this.color,
    required this.route,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: () => context.push(route),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Row(
            children: [
              Container(
                width: 56,
                height: 56,
                decoration: BoxDecoration(
                  color: color.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(icon, color: color, size: 28),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title,
                      style: const TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      subtitle,
                      style: TextStyle(
                        fontSize: 14,
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ],
                ),
              ),
              Icon(
                Icons.arrow_forward_ios,
                size: 16,
                color: Theme.of(context).colorScheme.outline,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

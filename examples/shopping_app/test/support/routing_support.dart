import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shopping_app/core/services/storage_service.dart';
import 'package:shopping_app/features/auth/models/auth_state.dart';
import 'package:shopping_app/features/auth/models/user.dart';
import 'package:shopping_app/features/auth/view_models/auth_view_model.dart';
import 'package:shopping_app/route.dart';

import 'fake_shopping_repository.dart';

Future<ShoppingRouter> shoppingRouter() async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await SharedPreferences.getInstance();
  final auth = AuthViewModel(
    AuthViewModelArgs(
      repository: FakeShoppingRepository(),
      storage: StorageService(prefs),
    ),
  );
  addTearDown(auth.dispose);
  return ShoppingRouter(auth: auth);
}

void authenticate(ShoppingRouter router, String username) {
  router.auth.value = AuthState(
    status: AuthStatus.authenticated,
    token: 'token-$username',
    user: user(username),
  );
}

void expireSession(ShoppingRouter router) {
  router.auth.value = const AuthState(status: AuthStatus.unauthenticated);
}

void setAuthInitial(ShoppingRouter router) {
  router.auth.value = const AuthState(status: AuthStatus.initial);
}

void setAuthLoading(ShoppingRouter router) {
  router.auth.value = const AuthState(status: AuthStatus.loading);
}

User user(String username) {
  return User(
    id: username.hashCode,
    email: '$username@example.com',
    username: username,
    name: Name(firstname: username, lastname: 'User'),
    phone: '555-0100',
  );
}

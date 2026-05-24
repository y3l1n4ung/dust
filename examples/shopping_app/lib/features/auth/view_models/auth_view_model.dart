import 'dart:convert';

import 'package:dust_state/dust_state.dart';

import '../../../core/data/shopping_repository.dart';
import '../../../core/logging/logger.dart';
import '../../../core/services/storage_service.dart';
import '../models/auth_state.dart';
import '../models/user.dart';

part 'auth_view_model.g.dart';

final class AuthViewModelArgs extends ViewModelArgs {
  const AuthViewModelArgs({
    required this.repository,
    required this.storage,
    super.observer,
  });

  final ShoppingRepository repository;
  final StorageService storage;
}

@ViewModel(
  state: AuthState,
  args: AuthViewModelArgs,
  initial: AuthState(status: AuthStatus.unauthenticated),
)
class AuthViewModel extends $AuthViewModel {
  AuthViewModel(super.args);

  @override
  Future<void> onInit() async {
    _restoreSession();
  }

  void _restoreSession() {
    try {
      final token = storage.getString(StorageService.authTokenKey);
      final userJson = storage.getString(StorageService.authUserKey);

      if (token != null && userJson != null) {
        final decoded = jsonDecode(userJson) as Map<String, dynamic>;
        final user = User.fromJson(Map<String, Object?>.from(decoded));
        logger.info('AUTH', 'Restored session for: ${user.username}');
        emit(
          AuthState(token: token, user: user, status: AuthStatus.authenticated),
        );
      }
    } catch (e) {
      logger.error('AUTH', 'Failed to restore session', e);
      emit(const AuthState(status: AuthStatus.unauthenticated));
    }
  }

  Future<void> _saveSession(String token, User user) async {
    await storage.setString(StorageService.authTokenKey, token);
    await storage.setString(
      StorageService.authUserKey,
      jsonEncode(user.toJson()),
    );
  }

  Future<void> _clearSession() async {
    await storage.remove(StorageService.authTokenKey);
    await storage.remove(StorageService.authUserKey);
  }

  Future<void> login(String username, String password) async {
    logger.userAction('login_attempt', {'username': username});
    emit(state.copyWith(status: AuthStatus.loading, errorMessage: null));

    try {
      final token = await repository.login(username, password);
      final user = await repository.getUser(1);

      await _saveSession(token, user);

      emit(
        state.copyWith(
          token: token,
          user: user,
          status: AuthStatus.authenticated,
        ),
      );
      logger.info('AUTH', 'Login successful for: ${user.username}');
    } catch (e) {
      logger.error('AUTH', 'Login failed', e);
      emit(
        state.copyWith(status: AuthStatus.error, errorMessage: e.toString()),
      );
    }
  }

  Future<void> register({
    required String email,
    required String username,
    required String password,
    required String firstName,
    required String lastName,
    required String phone,
  }) async {
    emit(state.copyWith(status: AuthStatus.loading, errorMessage: null));

    try {
      final userId = await repository.registerUser(
        email: email,
        username: username,
        password: password,
        firstName: firstName,
        lastName: lastName,
        phone: phone,
      );

      final user = User(
        id: userId,
        email: email,
        username: username,
        name: Name(firstname: firstName, lastname: lastName),
        phone: phone,
      );

      final token = 'registered_user_token_$userId';
      await _saveSession(token, user);

      emit(
        state.copyWith(
          token: token,
          user: user,
          status: AuthStatus.authenticated,
        ),
      );
      logger.info('AUTH', 'Registration successful for: $username');
    } catch (e) {
      logger.error('AUTH', 'Registration failed', e);
      emit(
        state.copyWith(status: AuthStatus.error, errorMessage: e.toString()),
      );
    }
  }

  Future<void> logout() async {
    logger.userAction('logout');
    await _clearSession();
    emit(const AuthState(status: AuthStatus.unauthenticated));
    logger.info('AUTH', 'User logged out');
  }
}

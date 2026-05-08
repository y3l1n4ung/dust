import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../models/user.dart';

part 'auth_view_model.ease.dart';

/// Authentication status
class AuthStatus {
  final User? user;
  final bool isLoading;
  final String? error;
  final bool isInitialized;

  const AuthStatus({
    this.user,
    this.isLoading = false,
    this.error,
    this.isInitialized = false,
  });

  bool get isAuthenticated => user != null;
  bool get isUnauthenticated => user == null && !isLoading;

  AuthStatus copyWith({
    User? user,
    bool? isLoading,
    String? error,
    bool? isInitialized,
    bool clearUser = false,
  }) {
    return AuthStatus(
      user: clearUser ? null : (user ?? this.user),
      isLoading: isLoading ?? this.isLoading,
      error: error,
      isInitialized: isInitialized ?? this.isInitialized,
    );
  }
}

/// Auth ViewModel - demonstrates navigation guards with persistence
@ease
class AuthViewModel extends StateNotifier<AuthStatus> {
  AuthViewModel() : super(const AuthStatus());

  static const _storageKey = 'auth_user';

  /// Initialize auth state from storage
  Future<void> initialize() async {
    if (state.isInitialized) return;

    state = state.copyWith(isLoading: true);

    try {
      final prefs = await SharedPreferences.getInstance();
      final userJson = prefs.getString(_storageKey);

      if (userJson != null) {
        final user = User.decode(userJson);
        state = state.copyWith(
          user: user,
          isLoading: false,
          isInitialized: true,
        );
      } else {
        state = state.copyWith(
          isLoading: false,
          isInitialized: true,
        );
      }
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        isInitialized: true,
        error: 'Failed to restore session',
      );
    }
  }

  /// Save user to storage
  Future<void> _saveUser(User user) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_storageKey, user.encode());
  }

  /// Clear user from storage
  Future<void> _clearUser() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_storageKey);
  }

  /// Login with email and password
  Future<void> login(String email, String password) async {
    state = state.copyWith(isLoading: true, error: null);

    // Simulate API call
    await Future.delayed(const Duration(seconds: 1));

    // Validation
    if (email.isEmpty || password.isEmpty) {
      state = state.copyWith(
        isLoading: false,
        error: 'Email and password required',
      );
      return;
    }

    if (password.length < 4) {
      state = state.copyWith(
        isLoading: false,
        error: 'Invalid credentials',
      );
      return;
    }

    // Success - create user
    final user = User(
      id: '1',
      name: email.split('@').first,
      email: email,
    );

    // Save to storage
    await _saveUser(user);

    state = state.copyWith(
      isLoading: false,
      user: user,
    );
  }

  /// Logout and clear storage
  Future<void> logout() async {
    await _clearUser();
    state = state.copyWith(clearUser: true);
  }
}

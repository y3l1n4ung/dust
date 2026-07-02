import 'package:dust_dart/serde.dart';

import 'user.dart';
part 'auth_state.g.dart';

/// Auth status values for the shopping app example.
enum AuthStatus {
  /// Initial auth status.
  initial,

  /// Loading auth status.
  loading,

  /// Authenticated auth status.
  authenticated,

  /// Unauthenticated auth status.
  unauthenticated,

  /// Error auth status.
  error,
}

/// Auth state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class AuthState with _$AuthState {
  /// User.
  final User? user;

  /// Token.
  final String? token;

  /// Status.
  final AuthStatus status;

  /// Error message.
  final String? errorMessage;

  /// Creates an [AuthState].
  const AuthState({
    this.user,
    this.token,
    this.status = AuthStatus.initial,
    this.errorMessage,
  });

  /// Whether the auth state is authenticated.
  bool get isAuthenticated => status == AuthStatus.authenticated;
}

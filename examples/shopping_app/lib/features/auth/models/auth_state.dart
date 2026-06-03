import 'package:dust_dart/serde.dart';

import 'user.dart';
part 'auth_state.g.dart';

enum AuthStatus { initial, loading, authenticated, unauthenticated, error }

@Derive([ToString(), CopyWith(), Eq()])
class AuthState with _$AuthState {
  final User? user;
  final String? token;
  final AuthStatus status;
  final String? errorMessage;

  const AuthState({
    this.user,
    this.token,
    this.status = AuthStatus.initial,
    this.errorMessage,
  });

  bool get isAuthenticated => status == AuthStatus.authenticated;
}

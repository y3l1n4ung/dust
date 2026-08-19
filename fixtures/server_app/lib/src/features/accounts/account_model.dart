import 'package:dust_dart/db.dart';
import 'package:dust_dart/serde.dart';

part 'account_model.g.dart';

/// One account, as stored.
///
/// No `Serialize`: this type carries a password hash and a salt, and a type
/// that cannot be serialized cannot be returned to a client by accident. See
/// [AccountView] for what is safe to send.
@Derive([ToString(), Eq(), FromRow()])
final class Account with _$Account {
  /// Creates an [Account].
  const Account({
    required this.id,
    required this.email,
    required this.passwordHash,
    required this.passwordSalt,
    required this.scopes,
  });

  /// The primary key.
  final int id;

  /// What they sign in with.
  final String email;

  /// PBKDF2 of the password, hex encoded.
  @Sqlx(rename: 'password_hash')
  final String passwordHash;

  /// The per-account salt, hex encoded.
  @Sqlx(rename: 'password_salt')
  final String passwordSalt;

  /// Comma-separated scopes.
  final String scopes;

  /// The scopes as a list.
  List<String> get grantedScopes =>
      scopes.isEmpty ? const [] : scopes.split(',');
}

/// What an account looks like on the wire.
@Derive([ToString(), Eq(), Serialize()])
final class AccountView with _$AccountView {
  /// Creates an [AccountView].
  const AccountView({
    required this.id,
    required this.email,
    required this.scopes,
  });

  /// Builds a view of [account].
  factory AccountView.of(Account account) => AccountView(
        id: account.id,
        email: account.email,
        scopes: account.grantedScopes,
      );

  /// The primary key.
  final int id;

  /// What they sign in with.
  final String email;

  /// What they may do.
  final List<String> scopes;
}

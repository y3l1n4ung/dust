import 'package:dust_server/server.dart';

/// Turns a bearer token into the caller it names, or rejects the request.
///
/// The token is `<id>|<scope>,<scope>` — a stand-in for a signed one, kept
/// readable so a test can write it by hand:
///
/// ```text
/// authorization: Bearer ada@dust.test|todos:read,todos:write
/// ```
///
/// This is the shape every custom extractor takes: implement
/// [FromRequestParts], put the configuration in the constructor, and let a
/// subclass with a zero-argument `const` constructor stand for a preconfigured
/// variant. `@Extract(TodosWrite)` names that subclass by type, which is how
/// axum selects an extractor too.
class BearerAuth implements FromRequestParts<AuthUser> {
  /// Requires [scope], when given.
  const BearerAuth({this.scope});

  /// The scope a token has to carry.
  final String? scope;

  @override
  Future<Result<AuthUser, Rejection>> extract(Request request) async {
    final raw = RequestParts.of(request).headers['authorization'];
    if (raw == null || !raw.startsWith('Bearer ')) {
      return const Err(Rejection.unauthorized('missing bearer token'));
    }

    final parts = raw.substring(7).split('|');
    if (parts.length != 2 || parts.first.isEmpty) {
      return const Err(Rejection.unauthorized('malformed bearer token'));
    }

    final user = AuthUser(parts.first, parts.last.split(','));
    if (scope != null && !user.can(scope!)) {
      return Err(Rejection.forbidden('requires scope $scope'));
    }
    return Ok(user);
  }
}

/// The write-scoped variant, configured by type the way axum does it.
final class TodosWrite extends BearerAuth {
  /// Requires the `todos:write` scope.
  const TodosWrite() : super(scope: 'todos:write');
}

/// Who is calling.
final class AuthUser {
  /// Creates a caller.
  const AuthUser(this.id, this.scopes);

  /// The caller's identifier, which is also what owns the todos they create.
  final String id;

  /// What the caller may do.
  final List<String> scopes;

  /// Whether this caller holds [scope].
  bool can(String scope) => scopes.contains(scope);

  /// Whether this caller may act on todos belonging to [owner].
  ///
  /// Your own always; someone else's only with `todos:admin`. Keeping the
  /// rule on the caller rather than in each handler means a new endpoint
  /// cannot forget it in a different way.
  bool mayActOn(String owner) => owner == id || can('todos:admin');
}

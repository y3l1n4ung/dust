import 'package:dust_server/server.dart';

// The hand-written half of the worked example in
// docs/design/server-plugin.md: the types an application author writes.

/// The value a custom extractor produces, with no framework coupling.
final class AuthUser {
  const AuthUser(this.id, this.scopes);

  final String id;
  final List<String> scopes;
}

/// An extractor carrying configuration, named at call sites as `@Extract`.
class BearerAuth implements FromRequestParts<AuthUser> {
  const BearerAuth({this.scope});

  final String? scope;

  @override
  Future<Result<AuthUser, Rejection>> extract(Request request) async {
    final raw = RequestParts.of(request).headers['authorization'];
    if (raw == null || !raw.startsWith('Bearer ')) {
      return const Err(Rejection.unauthorized('missing bearer token'));
    }

    final scopes = raw.substring(7).split(',');
    if (scope != null && !scopes.contains(scope)) {
      return Err(Rejection.forbidden('requires scope $scope'));
    }
    return Ok(AuthUser('u-1', scopes));
  }
}

/// The scoped variant, which is how configuration reaches `@Extract`.
final class TodosWrite extends BearerAuth {
  const TodosWrite() : super(scope: 'todos:write');
}

final class Todo {
  const Todo(this.id, this.title);

  final String id;
  final String title;

  Map<String, Object?> toJson() => {'id': id, 'title': title};
}

/// An error type carrying its own status, as the `Result` arm of a handler.
final class NotFound implements IntoResponse {
  const NotFound(this.message);

  final String message;

  @override
  Response intoResponse() => jsonResponse({'error': message}, status: 404);
}

final class CreateTodo {
  const CreateTodo(this.title);

  factory CreateTodo.fromJson(Map<String, Object?> json) {
    final title = json['title'];
    if (title is! String) throw const FormatException('title must be a string');
    return CreateTodo(title);
  }

  final String title;

  /// Stands in for the `Validate()` derive.
  Map<String, List<String>> validate() {
    return {
      if (title.isEmpty) 'title': ['must not be empty'],
    };
  }
}

import 'package:dust_server/server.dart';

import '../support.dart';
import 'models.dart';

// The class handler style. The annotated methods are exactly what an author
// writes; everything below the divider stands in for what the plugin will emit
// into todo_controller.g.dart, which is normally applied as a mixin.
//
// It shows extractors running in declaration order, short-circuiting on the
// first rejection, validation, response dispatch, and the guard.

@Controller('/todos', tags: ['todos'])
class TodoController {
  final _stored = <String, Todo>{'7': const Todo('7', 'existing')};

  @GET('/{id}', summary: 'Fetch one todo')
  Future<Result<Todo, NotFound>> get(
    @Extract(BearerAuth) AuthUser user,
    @Path() String id,
  ) async {
    final todo = _stored[id];
    return todo == null ? Err(NotFound('no todo $id')) : Ok(todo);
  }

  @POST('/', status: 201, summary: 'Create a todo')
  Future<Todo> create(
    @Extract(TodosWrite) AuthUser user,
    @Body() CreateTodo input,
  ) async {
    if (input.title == 'boom') throw StateError('repository exploded');
    if (input.title == 'existing') {
      throw const Rejection.conflict('duplicate title');
    }
    return Todo('8', input.title);
  }

  @DELETE('/{id}', status: 204)
  Future<void> remove(
    @Extract(TodosWrite) AuthUser user,
    @Path() String id,
  ) async {
    _stored.remove(id);
  }

  // -------------------------------------------------------------------------
  // Stand-in for todo_controller.g.dart.

  Router get routes => Router.module(
        prefix: '/todos',
        routes: [
          Route('GET', '/{id}', _handleGet),
          Route('POST', '/', _handleCreate),
          Route('DELETE', '/{id}', _handleRemove),
        ],
      );

  Future<Response> _handleGet(Request request) async {
    final user = await const BearerAuth().extract(request);
    if (user case Err(:final error)) return error.intoResponse();
    final user$ = (user as Ok<AuthUser, Rejection>).value;

    final id = await const PathExtractable<String>('id').extract(request);
    if (id case Err(:final error)) return error.intoResponse();
    final id$ = (id as Ok<String, Rejection>).value;

    return guard(() async {
      final result = await get(user$, id$);
      return switch (result) {
        Ok(:final value) => jsonResponse(value.toJson()),
        Err(:final error) => error.intoResponse(),
      };
    });
  }

  Future<Response> _handleCreate(Request request) async {
    final user = await const TodosWrite().extract(request);
    if (user case Err(:final error)) return error.intoResponse();
    final user$ = (user as Ok<AuthUser, Rejection>).value;

    final input = await const JsonExtractable<CreateTodo>(CreateTodo.fromJson)
        .extract(request);
    if (input case Err(:final error)) return error.intoResponse();
    final input$ = (input as Ok<CreateTodo, Rejection>).value;

    final errors = input$.validate();
    if (errors.isNotEmpty) {
      return Rejection.unprocessable(errors).intoResponse();
    }

    return guard(() async {
      final result = await create(user$, input$);
      return jsonResponse(result.toJson(), status: 201);
    });
  }

  Future<Response> _handleRemove(Request request) async {
    final user = await const TodosWrite().extract(request);
    if (user case Err(:final error)) return error.intoResponse();
    final user$ = (user as Ok<AuthUser, Rejection>).value;

    final id = await const PathExtractable<String>('id').extract(request);
    if (id case Err(:final error)) return error.intoResponse();
    final id$ = (id as Ok<String, Rejection>).value;

    return guard(() async {
      await remove(user$, id$);
      return noContent();
    });
  }
}

/// The composition an application writes by hand.
Handler todoApp({void Function(Object, StackTrace)? onError}) {
  final v1 = Router()..merge(TodoController().routes);
  return (Router(onError: onError)..nest('/api/v1', v1)).handler;
}

/// A request carrying a bearer token with [scopes].
Request authed(
  String method,
  String path, {
  String scopes = 'todos:read',
  Object? body,
  bool json = false,
}) {
  return request(
    method,
    path,
    headers: {
      'authorization': 'Bearer $scopes',
      if (json) 'content-type': 'application/json',
    },
    body: body,
  );
}

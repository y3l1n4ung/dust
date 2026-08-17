import 'package:dust_server/server.dart';

import '../models/create_todo.dart';
import '../models/todo.dart';
import 'auth.dart';
import 'store.dart';

/// Each endpoint declares what it produces, and the verb builder is generic
/// over that type, so returning the wrong model is a compile error rather than
/// a wrong response body.
///
/// An endpoint that cannot fail returns the value. One that can returns
/// `Result<T, Rejection>`, which keeps the failure in the signature instead of
/// in a comment.
///
/// Values come off the request directly, and every call throws the rejection
/// its extractor produced, so the first failure ends the endpoint. An
/// authenticating extractor returns the caller, which is what decides who sees
/// and touches what — a guard whose result is discarded is a guard that only
/// checks the easy half.

/// `GET /api/v1/todos`
///
/// Scoped to the caller. An admin sees everything; nobody else sees a todo
/// that is not theirs.
Future<List<Todo>> listTodos(Request request) async {
  final user = await request.extract<AuthUser>(const BearerAuth());
  final repository = await request.state<TodoStore>();
  final done = await request.query<bool?>('done');

  return repository.all(
    owner: user.can('todos:admin') ? null : user.id,
    done: done,
  );
}

/// `GET /api/v1/todos/{id}`
///
/// A todo the caller may not see answers 404 rather than 403: telling them it
/// exists is already more than they should learn.
Future<Result<Todo, Rejection>> readTodo(Request request) async {
  final user = await request.extract<AuthUser>(const BearerAuth());
  final id = await request.path<int>('id');
  final repository = await request.state<TodoStore>();

  final todo = await repository.find(id);
  if (todo == null || !user.mayActOn(todo.owner)) {
    return const Err(_missing);
  }
  return Ok(todo);
}

/// `POST /api/v1/todos`
///
/// `validBody` decodes with the generated `deserialize` and then runs the
/// generated `validate`. Both failures answer 422, and only the second fills
/// in `fields`: a body of the wrong shape has no field to blame.
///
/// Assigning to someone else needs `todos:admin`. That check needs the caller,
/// which is why the extractor's return value is used rather than dropped.
Future<Result<Todo, Rejection>> createTodo(Request request) async {
  final user = await request.extract<AuthUser>(const TodosWrite());
  final repository = await request.state<TodoStore>();
  final input = await request.validBody(CreateTodo.deserialize);

  if (!user.mayActOn(input.assignTo)) {
    return const Err(
      Rejection.forbidden('assigning to another user needs todos:admin'),
    );
  }
  return Ok(await repository.add(input));
}

/// `PATCH /api/v1/todos/{id}`
Future<Result<Todo, Rejection>> completeTodo(Request request) async {
  final user = await request.extract<AuthUser>(const TodosWrite());
  final id = await request.path<int>('id');
  final done = await request.query<bool>('done');
  final repository = await request.state<TodoStore>();

  final existing = await repository.find(id);
  if (existing == null || !user.mayActOn(existing.owner)) {
    return const Err(_missing);
  }
  return Ok((await repository.complete(id, done: done))!);
}

/// `DELETE /api/v1/todos/{id}`
///
/// `Null` on the success side is what answers 204: there is no body to send,
/// and saying so in the type keeps the empty answer deliberate.
Future<Result<Null, Rejection>> deleteTodo(Request request) async {
  final user = await request.extract<AuthUser>(const TodosWrite());
  final id = await request.path<int>('id');
  final repository = await request.state<TodoStore>();

  final existing = await repository.find(id);
  if (existing == null || !user.mayActOn(existing.owner)) {
    return const Err(_missing);
  }
  await repository.remove(id);
  return const Ok(null);
}

/// `GET /api/v1/me`
///
/// The caller, straight back. Nothing but the extractor's return value.
Future<Map<String, Object?>> whoAmI(Request request) async {
  final user = await request.extract<AuthUser>(const BearerAuth());

  return {'id': user.id, 'scopes': user.scopes};
}

/// `GET /health`
Future<Map<String, Object?>> health(Request request) async => {'ok': true};

const _missing = Rejection.notFound('no such todo');

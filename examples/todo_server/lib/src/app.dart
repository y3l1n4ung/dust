import 'dart:io';

import 'package:dust_server/server.dart';

import 'handlers.dart';
import 'store.dart';

/// Assembles the application, kept apart from `main` so tests can serve it.
///
/// [log] and [onError] are injected rather than hard-wired to `stdout` and
/// `stderr`, which is what lets a test read the access records and the escaped
/// errors instead of scraping the console.
Router buildApp(
  TodoStore repository, {
  void Function(AccessRecord)? log,
  void Function(Object error, StackTrace stack)? onError,
}) {
  final todos = Router()
    ..route('/', get(listTodos).post(createTodo, status: 201))
    ..route(
      '/{id}',
      get(readTodo).patch(completeTodo).delete(deleteTodo),
    );

  return Router(
    onError:
        onError ?? (error, stack) => stderr.writeln('handler failed: $error'),
  )
    ..layer(const RequestTimeout(Duration(seconds: 10)))
    ..layer(const RequestId())
    ..layer(AccessLog(log ?? (record) => stdout.writeln(record)))
    ..nest(
      '/api/v1',
      Router()
        ..nest('/todos', todos)
        ..route('/me', get(whoAmI)),
    )
    ..route('/health', get(health))
    ..withState(repository);
}

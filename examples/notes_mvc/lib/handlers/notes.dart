import 'package:dust_dart/db.dart';
import 'package:dust_server/server.dart';

import '../db/notes_queries.dart';

/// One function per route, and nothing between it and the data.
///
/// No repository, no store interface, no row type separate from the model. The
/// generated queries **are** the data layer, and they arrive the same way
/// everything else does: `request.state<Notes>()`, which is axum's `State<S>`
/// and FastAPI's dependency in Dart. Nothing is captured in a closure or held
/// in a field, so a handler is a plain function a generator could have written.

/// `GET /notes`
Future<List<Note>> index(Request request) async {
  final notes = await request.state<Notes>();

  return _ok(await notes.listAll());
}

/// `GET /notes/{id}`
Future<Result<Note, Rejection>> show(Request request) async {
  final id = await request.path<int>('id');
  final notes = await request.state<Notes>();

  final note = _ok(await notes.findById(id));
  return note == null ? const Err(_missing) : Ok(note);
}

/// `POST /notes`
Future<Note> create(Request request) async {
  final notes = await request.state<Notes>();
  final draft = await request.validBody(NoteDraft.deserialize);

  final inserted = _ok(await notes.insert(draft.title, draft.body));
  return Note(id: inserted.lastInsertId!, title: draft.title, body: draft.body);
}

/// `PUT /notes/{id}`
Future<Result<Note, Rejection>> replace(Request request) async {
  final id = await request.path<int>('id');
  final notes = await request.state<Notes>();
  final draft = await request.validBody(NoteDraft.deserialize);

  final updated = _ok(await notes.update(draft.title, draft.body, id));
  return updated.rowsAffected == 0
      ? const Err(_missing)
      : Ok(Note(id: id, title: draft.title, body: draft.body));
}

/// `DELETE /notes/{id}`
Future<Result<Null, Rejection>> destroy(Request request) async {
  final id = await request.path<int>('id');
  final notes = await request.state<Notes>();

  final deleted = _ok(await notes.deleteById(id));
  return deleted.rowsAffected == 0 ? const Err(_missing) : const Ok(null);
}

/// `GET /health`
Future<Map<String, Object?>> health(Request request) async => {'ok': true};

/// Takes the value, or answers for the database failure.
///
/// A driver message names columns and constraints, so it goes to the error
/// sink and the client gets an opaque 500.
T _ok<T>(Result<T, SqlxError> outcome) => switch (outcome) {
      Ok(:final value) => value,
      Err(:final error) => throw _report(error),
    };

Rejection _report(SqlxError error) {
  ServerErrors.report(error, StackTrace.current);
  return const Rejection.internal();
}

const _missing = Rejection.notFound('no such note');

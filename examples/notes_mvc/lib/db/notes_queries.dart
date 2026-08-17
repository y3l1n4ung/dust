import 'package:dust_dart/db.dart';

import '../models/note.dart';

export '../models/note.dart';

part 'notes_queries.g.dart';

/// Every query the API makes. There is no repository in front of this.
///
/// Bound to a [DatabaseExecutor], so the same queries run on a connection or
/// inside a transaction: `Notes(database.connection)` normally, `Notes(tx)`
/// when several statements have to happen together.
@SqlxDao()
abstract final class Notes {
  /// Binds the queries to [db].
  const factory Notes(DatabaseExecutor db) = _$Notes;

  /// Every note.
  @Query(r'SELECT id, title, body FROM notes ORDER BY id')
  Future<Result<List<Note>, SqlxError>> listAll();

  /// One note, or nothing.
  @Query(r'SELECT id, title, body FROM notes WHERE id = $1')
  Future<Result<Note?, SqlxError>> findById(int id);

  /// Stores a note.
  @Query(r'INSERT INTO notes (title, body) VALUES ($1, $2)')
  Future<Result<ExecResult, SqlxError>> insert(String title, String body);

  /// Replaces a note's text.
  @Query(r'UPDATE notes SET title = $1, body = $2 WHERE id = $3')
  Future<Result<ExecResult, SqlxError>> update(
    String title,
    String body,
    int id,
  );

  /// Removes a note.
  @Query(r'DELETE FROM notes WHERE id = $1')
  Future<Result<ExecResult, SqlxError>> deleteById(int id);
}

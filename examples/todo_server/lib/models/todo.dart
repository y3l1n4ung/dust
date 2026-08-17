import 'package:dust_dart/db.dart';
import 'package:dust_dart/serde.dart';

part 'todo.g.dart';

/// One stored to-do.
///
/// One class, three jobs: it is the row `FromRow` maps out of the database,
/// the JSON `Serialize` writes to the wire, and the value `Deserialize` reads
/// back. A separate row type earns its keep once the table and the resource
/// start to differ — not before.
///
/// No `@Sqlx(renameAll:)`: every column here is one lowercase word, so the
/// rename would do nothing. It also currently stops the DAO from accepting
/// this type as a result — see the note in `models/todo_database.dart`.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize(), FromRow()])
final class Todo with _$Todo {
  /// Creates a [Todo].
  const Todo({
    required this.id,
    required this.title,
    required this.owner,
    required this.done,
  });

  /// Reads a [Todo] from decoded JSON.
  static Todo deserialize(Map<String, Object?> json) => _$TodoDeserialize(json);

  /// The primary key, which is also what appears in paths.
  final int id;

  /// What to do.
  final String title;

  /// Who it belongs to, taken from the authenticated caller rather than from
  /// the request body, so a client cannot claim someone else's todo.
  final String owner;

  /// Whether it is finished.
  final bool done;
}

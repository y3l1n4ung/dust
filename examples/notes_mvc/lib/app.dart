import 'package:dust_server/server.dart';

import 'handlers/notes.dart' as notes;
import 'db/notes_queries.dart';

export 'handlers/notes.dart';
export 'db/database.dart';
export 'db/notes_queries.dart';

/// The whole application: five routes and one piece of state.
///
/// The queries go in with `withState`, so every handler reads them the same
/// way it reads a path parameter. Swapping the database — a file, a fake, a
/// transaction — is a change here and nowhere else.
///
/// ```dart
/// final database = NotesDatabase.open('notes.db');
/// final app = buildApp(Notes(database.connection));
/// ```
Router buildApp(Notes queries) {
  return Router()
    ..layer(const RequestId())
    ..nest(
      '/notes',
      Router()
        ..route('/', get(notes.index).post(notes.create, status: 201))
        ..route(
          '/{id}',
          get(notes.show).put(notes.replace).delete(notes.destroy),
        ),
    )
    ..route('/health', get(notes.health))
    ..withState(queries);
}

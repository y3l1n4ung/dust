import 'package:dust_server/server.dart';

import 'models.dart';

// The other handler style: top-level functions with no controller class.
//
// The annotated functions are exactly what an author writes. Everything below
// the divider stands in for what the plugin will emit into notes.g.dart: one
// handler per function, and a top-level function exposing the group.
//
// A function rather than a getter: a Router is mutable and is sealed when
// its handler is read, so two servers in one isolate each need their own —
// and a getter makes `noteRoutes.withState(x)` compile, configure a
// throwaway, and answer 500 claiming the state was never attached.
//
// Dependencies cannot live in fields here, so they arrive as state attached
// where the routes are mounted, the way axum's `State` pairs with
// `with_state`.

/// What a controller class would have held in a field.
final class NoteStore {
  NoteStore([Map<String, String>? seed]) : notes = {...?seed};

  final Map<String, String> notes;
}

@GET('/', summary: 'List notes')
Future<List<String>> list(
  @Extract(BearerAuth) AuthUser user,
  @State() NoteStore store,
) async {
  return store.notes.values.toList()..sort();
}

@GET('/{id}')
Future<Result<String, NotFound>> read(
  @Path() String id,
  @State() NoteStore store,
) async {
  final note = store.notes[id];
  return note == null ? Err(NotFound('no note $id')) : Ok(note);
}

@POST('/', status: 201)
Future<String> write(
  @Extract(BearerAuth) AuthUser user,
  @State() NoteStore store,
  @RawBody() String body,
) async {
  store.notes['${store.notes.length + 1}'] = body;
  return body;
}

// ---------------------------------------------------------------------------
// Stand-in for notes.g.dart.

Router noteRoutes() => Router.module(
      prefix: '',
      routes: [
        Route('GET', '/', _handleList),
        Route('GET', '/{id}', _handleRead),
        Route('POST', '/', _handleWrite),
      ],
    );

Future<Response> _handleList(Request request) async {
  final user = await const BearerAuth().extract(request);
  if (user case Err(:final error)) return error.intoResponse();
  final user$ = (user as Ok<AuthUser, Rejection>).value;

  final store = await const StateExtractable<NoteStore>().extract(request);
  if (store case Err(:final error)) return error.intoResponse();
  final store$ = (store as Ok<NoteStore, Rejection>).value;

  return guard(() async => jsonResponse(await list(user$, store$)));
}

Future<Response> _handleRead(Request request) async {
  final id = await const PathExtractable<String>('id').extract(request);
  if (id case Err(:final error)) return error.intoResponse();
  final id$ = (id as Ok<String, Rejection>).value;

  final store = await const StateExtractable<NoteStore>().extract(request);
  if (store case Err(:final error)) return error.intoResponse();
  final store$ = (store as Ok<NoteStore, Rejection>).value;

  return guard(() async {
    final result = await read(id$, store$);
    return switch (result) {
      Ok(:final value) => jsonResponse({'note': value}),
      Err(:final error) => error.intoResponse(),
    };
  });
}

Future<Response> _handleWrite(Request request) async {
  final user = await const BearerAuth().extract(request);
  if (user case Err(:final error)) return error.intoResponse();
  final user$ = (user as Ok<AuthUser, Rejection>).value;

  final store = await const StateExtractable<NoteStore>().extract(request);
  if (store case Err(:final error)) return error.intoResponse();
  final store$ = (store as Ok<NoteStore, Rejection>).value;

  final body = await const TextBodyExtractable().extract(request);
  if (body case Err(:final error)) return error.intoResponse();
  final body$ = (body as Ok<String, Rejection>).value;

  return guard(() async {
    final written = await write(user$, store$, body$);
    return jsonResponse({'note': written}, status: 201);
  });
}

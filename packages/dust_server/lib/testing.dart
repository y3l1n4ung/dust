/// Test utilities for dust_server.
///
/// ```dart
/// import 'package:dust_server/testing.dart';
///
/// void main() {
///   test('lists todos', () async {
///     final client = TestClient(buildApp());
///     addTearDown(client.close);
///
///     (await client.get('/todos').send())
///         ..assertOk()
///         ..assertJsonContains({'count': 0});
///   });
/// }
/// ```
///
/// - `TestClient(router)` — in-process, no socket, sub-millisecond.
/// - `TestClient.serve(router)` — real HTTP on port 0, catches wire bugs.
///
/// Separate from `server.dart` so production code never pulls test utilities.
library;

export 'src/testing/testing.dart';

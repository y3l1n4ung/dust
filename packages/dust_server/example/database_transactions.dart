import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:dust_server/server.dart';

/// A request that must be all-or-nothing.
///
/// Checkout is the shape: reserve the stock, then write the order. Two writes,
/// and a request that does the first and fails the second has taken the item
/// off the shelf for an order that does not exist.
///
/// `transaction` decides on the closure's return value — `Ok` commits, `Err`
/// rolls back — so there is no `commit()` to forget and no path out that skips
/// the decision. A throw rolls back too.
///
/// The order of the two writes is the other half. Stock is reserved **first**,
/// so a sold-out item fails before an order exists. Insert the order first and
/// a rollback is the only thing standing between you and a paid order you
/// cannot ship.
///
/// ```shell
/// dart run example/database_transactions.dart
///
/// curl -X POST localhost:8080/checkout -H 'content-type: application/json' \
///   -d '{"item":"shirt","quantity":2}'
/// curl -X POST localhost:8080/checkout -H 'content-type: application/json' \
///   -d '{"item":"shirt","quantity":99}'   # 409, and nothing is reserved
/// curl localhost:8080/stock
/// ```
Future<void> main() async {
  final database = openDatabase();

  final server = await serve(buildApp(database), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await database.close();
  await server.close(drain: const Duration(seconds: 5));
}

/// Opens the database, with one item on the shelf to sell.
///
/// The `CHECK` is the last line of defence: the database refuses an oversell
/// even if every code path above it forgets to.
Sqlite3Driver openDatabase() {
  final database = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(foreignKeys: true),
    migrations: const <String, String>{
      '0001_create_shop.sql': '''
CREATE TABLE stock (
  item    TEXT PRIMARY KEY,
  on_hand INTEGER NOT NULL CHECK (on_hand >= 0)
);
CREATE TABLE orders (
  id       INTEGER PRIMARY KEY,
  item     TEXT NOT NULL,
  quantity INTEGER NOT NULL
);
''',
    },
  );
  database.execute(
    r'INSERT INTO stock (item, on_hand) VALUES ($1, $2)',
    const <Object?>['shirt', 5],
  );
  return database;
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(Sqlite3Driver database) {
  return Router()
    ..route('/checkout', post(checkout, status: 201))
    ..route('/stock', get(listStock))
    ..withState(database);
}

/// `POST /checkout` — reserve the stock and write the order, or neither.
Future<Result<Map<String, Object?>, Rejection>> checkout(
  Request request,
) async {
  final database = await request.state<Sqlite3Driver>();
  final body = await request.body<Map<String, Object?>>((json) => json);
  final item = body['item']! as String;
  final quantity = body['quantity']! as int;

  final placed = await database.transaction<Map<String, Object?>>((tx) async {
    // Reserve first. The WHERE is what makes this safe under concurrency: two
    // requests for the last item cannot both match it.
    final reserved = await tx.execute(
      r'UPDATE stock SET on_hand = on_hand - $1 '
      r'WHERE item = $2 AND on_hand >= $1',
      <Object?>[quantity, item],
    );
    if (reserved case Err(:final error)) {
      return Err<Map<String, Object?>, SqlxError>(error);
    }
    if (reserved.unwrapOrElse((error) => throw error).rowsAffected == 0) {
      // Returning Err is what rolls this back. Nothing was written yet, but
      // the next request in this closure would have been.
      return Err<Map<String, Object?>, SqlxError>(
        SqlxError.query('not enough $item', operation: 'checkout'),
      );
    }

    return tx.fetchOne<Map<String, Object?>>(
      r'INSERT INTO orders (item, quantity) VALUES ($1, $2) '
      r'RETURNING id, item, quantity',
      <Object?>[item, quantity],
      (row) => <String, Object?>{
        'id': row.read<int>('id'),
        'item': row.read<String>('item'),
        'quantity': row.read<int>('quantity'),
      },
    );
  });

  return switch (placed) {
    Ok(:final value) => Ok(value),
    // "Somebody bought the last one" is an ordinary outcome of a shop, not a
    // malformed request — 409 rather than 422.
    Err() => const Err(Rejection.conflict('not enough stock')),
  };
}

/// `GET /stock` — what is left, so a rollback can be seen.
Future<List<Map<String, Object?>>> listStock(Request request) async {
  final database = await request.state<Sqlite3Driver>();

  final stock = await database.fetchAll<Map<String, Object?>>(
    'SELECT item, on_hand FROM stock ORDER BY item',
    const <Object?>[],
    (row) => <String, Object?>{
      'item': row.read<String>('item'),
      'onHand': row.read<int>('on_hand'),
    },
  );

  return stock.unwrapOrElse((error) => throw error);
}

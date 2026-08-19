import 'dart:convert';

import 'package:dust_dart/db.dart';
import 'package:dust_server/server.dart';

import '../accounts/account_model.dart';
import '../accounts/require_scope.dart';
import '../orders/order_model.dart';
import '../orders/orders_repo.dart';

part 'exports_router.g.dart';

// The feature that streams. An export is unbounded by nature, so it is written
// out as it is produced rather than assembled and then sent.

/// `GET /orders.csv` — this account's orders, streamed.
///
/// Returning a `Stream<List<int>>` is what makes it a stream: the runtime sends
/// each chunk as it arrives and sets no `content-length`, because the length is
/// not known until the work finishes. Collecting the rows into a string first
/// would hold the whole export in memory for no benefit.
@GET('/orders.csv', summary: 'Export your orders')
Future<Response> exportOrders(
  @Extract(RequireScope) Account account,
  @State() OrdersRepo repo,
) async {
  return streamed(
    _rows(account, repo),
    contentType: 'text/csv; charset=utf-8',
    headers: const {
      // Names the download rather than letting a browser guess from the URL.
      'content-disposition': 'attachment; filename="orders.csv"',
    },
  );
}

/// Emits the header, then a page of rows at a time.
Stream<List<int>> _rows(Account account, OrdersRepo repo) async* {
  yield utf8.encode('id,item,quantity,placed_at\n');

  const pageSize = 100;
  var offset = 0;

  while (true) {
    final page = await repo.pageFor(account.id, pageSize, offset);
    if (page case Err()) return;

    final rows = (page as Ok<List<Order>, SqlxError>).value;
    if (rows.isEmpty) return;

    for (final order in rows) {
      // The item is quoted and its quotes doubled: a value containing a comma
      // would otherwise become two columns, and one containing a quote would
      // break the row.
      final item = order.item.replaceAll('"', '""');
      yield utf8.encode(
        '${order.id},"$item",${order.quantity},${order.placedAt}\n',
      );
    }

    if (rows.length < pageSize) return;
    offset += pageSize;
  }
}

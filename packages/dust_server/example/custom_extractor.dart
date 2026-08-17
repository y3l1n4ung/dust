import 'dart:io';

import 'package:dust_server/server.dart';

/// Writing an extractor of your own.
///
/// Implement `FromRequestParts<T>` when the value comes from the head of the
/// request — headers, path, query, cookies. Implement `FromRequest<T>` when it
/// needs the body, because the body can be read only once and the type says so.
///
/// > **The one rule that matters.** Inside `extract`, use the extractor
/// > **classes**, which return `Err`. Never `request.header(...)`: the shortcuts
/// > *throw* the rejection, and a throw sails straight past whatever was
/// > composing your extractor — `FirstOf` cannot try the next one, and
/// > `optional` cannot turn it into `None`. Returning `Err` is what makes an
/// > extractor composable.
///
/// Run it with `dart run example/custom_extractor.dart`:
///
/// ```bash
/// curl -s localhost:8080/page -H 'x-page-size: 25'
/// curl -s localhost:8080/page                       # the default
/// curl -i localhost:8080/page -H 'x-page-size: 500' # 400, over the cap
/// curl -i localhost:8080/page -H 'x-page-size: big' # 400, not a number
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() => Router()..route('/page', get(readPage));

/// `GET /page`
Future<Map<String, Object?>> readPage(Request request) async {
  final size = await request.extract(const PageSizeExtractor());

  return {'size': size.value};
}

/// How many rows a client asked for, validated once.
final class PageSize {
  /// Creates a [PageSize].
  const PageSize._(this.value);

  /// The number of rows.
  final int value;
}

/// Reads `x-page-size`, defaulted and capped.
///
/// The cap is the reason this is an extractor rather than a line in each
/// handler: `?limit=1000000` is a denial-of-service request dressed as
/// pagination, and one place to refuse it beats twenty.
final class PageSizeExtractor implements FromRequestParts<PageSize> {
  /// Reads the header, falling back to [fallback] and refusing over [max].
  const PageSizeExtractor({this.fallback = 10, this.max = 100});

  /// What an absent header means.
  final int fallback;

  /// The largest value accepted.
  final int max;

  @override
  Future<Result<PageSize, Rejection>> extract(Request request) async {
    // The class, not `request.header(...)`: this returns Err, which composes.
    // It is also generic and coerces, so `<int?>` turns "big" into a 400 here
    // rather than needing a tryParse below.
    final header =
        await const HeaderExtractable<int?>('x-page-size').extract(request);

    switch (header) {
      case Err(:final error):
        return Err(error);
      case Ok(value: null):
        return Ok(PageSize._(fallback));
      case Ok(value: final int asked):
        if (asked < 1) {
          return const Err(
            Rejection.badRequest('x-page-size must be a positive integer'),
          );
        }
        if (asked > max) {
          return Err(Rejection.badRequest('x-page-size may not exceed $max'));
        }
        return Ok(PageSize._(asked));
    }
  }
}

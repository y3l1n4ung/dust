import 'dart:io';

import 'package:dust_dart/derive.dart';
import 'package:dust_server/server.dart';

/// Answering 422 with the errors for every field, not just the first.
///
/// `validBody(Model.deserialize)` decodes **and** validates. Two different
/// failures land on the same status and the same body shape:
///
/// * JSON of the wrong shape — a missing field, a string where a number belongs
/// * a value that decoded fine and broke a rule — an empty title, a price of 0
///
/// One format for both is the point. A client writes one error renderer.
///
/// A generated model gets `validate()` from `@Derive([Validate()])`; this one
/// implements it by hand so the example runs with no codegen step. Either way
/// the endpoint is the same line.
///
/// Run it with `dart run example/validation_422.dart`:
///
/// ```bash
/// curl -s -X POST localhost:8080/products -H 'content-type: application/json' \
///   -d '{"title":"Tee","priceCents":2500}'
/// curl -s -X POST localhost:8080/products -H 'content-type: application/json' \
///   -d '{"title":"","priceCents":0}'     # 422, both fields
/// curl -s -X POST localhost:8080/products -H 'content-type: application/json' \
///   -d '{"priceCents":2500}'             # 422, the shape
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() =>
    Router()..route('/products', post(createProduct, status: 201));

/// `POST /products`
///
/// One line, and the 422 never reaches it — `validBody` answers first.
Future<Map<String, Object?>> createProduct(Request request) async {
  final product = await request.validBody(NewProduct.deserialize);

  return {'title': product.title, 'priceCents': product.priceCents};
}

/// What the endpoint accepts.
final class NewProduct implements Validatable {
  /// Creates a [NewProduct].
  const NewProduct({required this.title, required this.priceCents});

  /// Reads a [NewProduct] from decoded JSON.
  ///
  /// The message on this throw reaches the client as the 422 body, so it names
  /// the field. A bare `json['title']! as String` would answer "Null check
  /// operator used on a null value" instead, which is an implementation detail
  /// leaking out of the server.
  static NewProduct deserialize(Map<String, Object?> json) => NewProduct(
        title: switch (json['title']) {
          final String title => title,
          null => throw const FormatException('title is required'),
          _ => throw const FormatException('title must be a string'),
        },
        priceCents: switch (json['priceCents']) {
          final int cents => cents,
          null => throw const FormatException('priceCents is required'),
          _ => throw const FormatException('priceCents must be an integer'),
        },
      );

  /// The name on the product page.
  final String title;

  /// The price, in cents. Integers, because money that does not add up is the
  /// one bug a shop cannot ship.
  final int priceCents;

  @override
  ValidationResult validate() {
    // Every rule runs. Returning on the first failure would tell a client to
    // fix one field, then reject the next request for the field beside it.
    final errors = <ValidationError>[
      if (title.trim().isEmpty)
        const ValidationError(field: 'title', message: 'is required'),
      if (priceCents <= 0)
        const ValidationError(
          field: 'priceCents',
          message: 'must be more than zero',
        ),
    ];

    return errors.isEmpty ? const Valid() : Invalid(errors);
  }

  @override
  void validateOrThrow() {
    if (validate() case final Invalid invalid) {
      throw ValidationException(invalid.errors);
    }
  }
}

import 'package:dust_server/server.dart';

import 'caller.dart';

/// A custom extractor carrying configuration, named at call sites as
/// `@Extract(RequireScope)`.
final class RequireScope implements FromRequestParts<Caller> {
  /// Requires [scope], or any credential when it is null.
  const RequireScope([this.scope]);

  /// The scope a caller must hold.
  final String? scope;

  @override
  Future<Result<Caller, Rejection>> extract(Request request) async {
    switch (await const BearerTokenExtractable().extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final token):
        final scopes = token.split(',');
        final required = scope;
        if (required != null && !scopes.contains(required)) {
          return Err(Rejection.forbidden('requires scope $required'));
        }
        return Ok(Caller('u-1', scopes));
    }
  }
}

/// The scoped variant, which is how configuration reaches `@Extract`.
final class OrdersWrite extends RequireScope {
  /// Requires `orders:write`.
  const OrdersWrite() : super('orders:write');
}

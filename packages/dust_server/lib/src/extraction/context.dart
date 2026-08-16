import 'dart:io' show HttpConnectionInfo;

import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../response/rejection.dart';
import 'extractable.dart';

/// Extracts a value middleware stored in the shelf request context.
///
/// For values middleware stored under a name of its own. Application state
/// should use `@State()` and `withState` instead, which key by type. A missing
/// or wrongly typed value is a 500, since it means the middleware did not run
/// or wrote something else.
final class ContextExtractable<T> implements FromRequestParts<T> {
  /// Reads the context entry named [key].
  const ContextExtractable(this.key);

  /// The context key.
  final String key;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final value = request.context[key];
    if (value == null) {
      if (null is T) return Ok(null as T);
      return Err(Rejection.internal('context value "$key" is missing'));
    }
    if (value is! T) {
      return Err(Rejection.internal('context value "$key" is not a $T'));
    }
    return Ok(value as T);
  }
}

/// The peer address and port of the connection serving this request.
final class PeerInfo {
  /// Creates peer information.
  const PeerInfo({
    required this.remoteAddress,
    required this.remotePort,
    required this.localPort,
  });

  /// The remote address, as text.
  final String remoteAddress;

  /// The remote port.
  final int remotePort;

  /// The local port the request arrived on.
  final int localPort;

  @override
  String toString() => 'PeerInfo($remoteAddress:$remotePort)';
}

/// Extracts the connection information for this request.
///
/// `shelf_io` stores it under `shelf.io.connection_info`. Requests built in
/// tests, or served by another adapter, reject with 500 because there is no
/// connection to report on.
final class PeerExtractable implements FromRequestParts<PeerInfo> {
  /// Reads the connection information.
  const PeerExtractable();

  /// The context key `shelf_io` writes connection information to.
  static const contextKey = 'shelf.io.connection_info';

  @override
  Future<Result<PeerInfo, Rejection>> extract(Request request) async {
    final info = request.context[contextKey];
    if (info is PeerInfo) return Ok(info);
    if (info is HttpConnectionInfo) {
      return Ok(
        PeerInfo(
          remoteAddress: info.remoteAddress.address,
          remotePort: info.remotePort,
          localPort: info.localPort,
        ),
      );
    }
    return const Err(
      Rejection.internal('connection information is unavailable'),
    );
  }
}

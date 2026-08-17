import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../router/middleware.dart';

/// Which origins a browser may call this server from.
///
/// A browser sends `Origin` and refuses the response unless the server names
/// that origin back. Everything else in CORS follows from that one exchange.
sealed class AllowedOrigins {
  const AllowedOrigins();

  /// Any origin at all, answered as `*`.
  ///
  /// Cannot be combined with credentials: the specification forbids `*` on a
  /// credentialed request, and a browser will refuse the response.
  const factory AllowedOrigins.any() = _AnyOrigin;

  /// Exactly these origins, compared case-sensitively as the specification
  /// requires — `https://Example.com` is not `https://example.com`.
  const factory AllowedOrigins.only(Set<String> origins) = _ListedOrigins;

  /// Whatever [predicate] accepts, for a rule a list cannot express.
  const factory AllowedOrigins.matching(
      bool Function(String origin) predicate) = _MatchingOrigins;

  /// What to answer for [origin], or `null` to refuse it.
  String? headerFor(String? origin);

  /// Whether the answer depends on the request, and so needs `Vary: Origin`.
  bool get varies;
}

final class _AnyOrigin extends AllowedOrigins {
  const _AnyOrigin();

  @override
  String? headerFor(String? origin) => '*';

  @override
  bool get varies => false;
}

final class _ListedOrigins extends AllowedOrigins {
  const _ListedOrigins(this.origins);

  final Set<String> origins;

  @override
  String? headerFor(String? origin) =>
      origin != null && origins.contains(origin) ? origin : null;

  @override
  bool get varies => true;
}

final class _MatchingOrigins extends AllowedOrigins {
  const _MatchingOrigins(this.predicate);

  final bool Function(String origin) predicate;

  @override
  String? headerFor(String? origin) =>
      origin != null && predicate(origin) ? origin : null;

  @override
  bool get varies => true;
}

/// Answers the cross-origin questions a browser asks before it will hand a
/// response to JavaScript.
///
/// ```dart
/// final app = Router()
///   ..layer(Cors(origins: AllowedOrigins.only({'https://app.example'})))
///   ..merge(routes);
/// ```
///
/// Two kinds of request arrive. A **preflight** is `OPTIONS` carrying
/// `Access-Control-Request-Method`; it is answered here and the handler never
/// runs, because the browser is asking permission rather than making the call.
/// Everything else runs normally and gets the allow headers added.
///
/// Put it at the top of the router. A layer below the one that rejects a
/// request never runs, and a 401 without CORS headers reaches the browser as
/// an opaque network error rather than as "you are not signed in".
final class Cors implements Layer {
  /// Allows [origins], with the usual defaults for everything else.
  Cors({
    this.origins = const AllowedOrigins.any(),
    this.methods = const {'GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'},
    this.headers = const {'accept', 'authorization', 'content-type'},
    this.exposeHeaders = const {},
    this.credentials = false,
    this.maxAge,
  }) {
    if (credentials && origins.headerFor('https://example.test') == '*') {
      throw ArgumentError(
        'CORS credentials cannot be used with AllowedOrigins.any(): a browser '
        'refuses a credentialed response whose allowed origin is "*". Name the '
        'origins with AllowedOrigins.only or .matching instead.',
      );
    }
  }

  /// Which origins may call.
  final AllowedOrigins origins;

  /// Which methods a preflight may ask for.
  final Set<String> methods;

  /// Which request headers a preflight may ask for.
  final Set<String> headers;

  /// Which response headers JavaScript may read beyond the safelisted ones.
  final Set<String> exposeHeaders;

  /// Whether cookies and `Authorization` may be sent.
  final bool credentials;

  /// How long a browser may cache the preflight answer.
  final Duration? maxAge;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final parts = RequestParts.of(request);
        final origin = parts.headers['origin'];
        final allowed = origins.headerFor(origin);

        // An `OPTIONS` without `Access-Control-Request-Method` is an ordinary
        // request asking what a path supports, not a preflight.
        final isPreflight = parts.method == 'OPTIONS' &&
            parts.headers['access-control-request-method'] != null;

        if (isPreflight) {
          // A refused origin still gets an answer, just without permission.
          return Response(
            204,
            headers: {
              ...?_allowHeaders(allowed),
              if (allowed != null) ...{
                'access-control-allow-methods': methods.join(', '),
                'access-control-allow-headers': headers.join(', '),
                if (maxAge != null)
                  'access-control-max-age': '${maxAge!.inSeconds}',
              },
              ..._varyPreflight(),
            },
          );
        }

        final response = await inner(request);
        return response.change(headers: _allowHeaders(allowed) ?? _vary());
      };
    };
  }

  /// The headers that grant access, or `null` when the origin is not allowed.
  ///
  /// A refused origin gets no `Access-Control-Allow-Origin` at all rather than
  /// a wrong one — the browser then blocks the response, which is the point.
  Map<String, String>? _allowHeaders(String? allowed) {
    if (allowed == null) return null;
    return {
      'access-control-allow-origin': allowed,
      if (credentials) 'access-control-allow-credentials': 'true',
      if (exposeHeaders.isNotEmpty)
        'access-control-expose-headers': exposeHeaders.join(', '),
      ..._vary(),
    };
  }

  /// `Vary: Origin` whenever the answer depends on the request.
  ///
  /// Without it a shared cache can hand one origin's allowance to another,
  /// which is a real hole rather than a tidiness point.
  Map<String, String> _vary() =>
      origins.varies ? const {'vary': 'Origin'} : const {};

  Map<String, String> _varyPreflight() => origins.varies
      ? const {
          'vary': 'Origin, Access-Control-Request-Method, '
              'Access-Control-Request-Headers',
        }
      : const {};
}

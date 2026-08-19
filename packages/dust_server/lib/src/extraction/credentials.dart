import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'cookie.dart';
import 'extractable.dart';

/// Extracts an API key from a header, falling back to the query string.
///
/// Machine-to-machine calls, and the one scheme where a key in the URL is
/// still common — which is also why it ends up in access logs, proxy logs, and
/// browser history. The header wins when both are present, and [allowQuery]
/// turns the fallback off for a deployment that cannot afford it.
final class ApiKeyExtractable implements FromRequestParts<String> {
  /// Reads the key from [header], or from the [query] parameter.
  const ApiKeyExtractable({
    this.header = 'x-api-key',
    this.query = 'api_key',
    this.allowQuery = true,
  });

  /// The header carrying the key.
  final String header;

  /// The query parameter carrying the key, used only when [allowQuery].
  final String query;

  /// Whether a key in the query string is accepted at all.
  final bool allowQuery;

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    final fromHeader = RequestParts.of(request).headers[header.toLowerCase()];
    if (fromHeader != null && fromHeader.isNotEmpty) return Ok(fromHeader);

    if (allowQuery) {
      final fromQuery = request.url.queryParameters[query];
      if (fromQuery != null && fromQuery.isNotEmpty) return Ok(fromQuery);
    }

    return Err(
      Rejection.unauthorized(
        'expected an API key in the $header header',
        challenge: 'ApiKey',
      ),
    );
  }
}

/// Extracts a session identifier from a cookie.
///
/// The browser half of authentication. It rejects with 401 rather than the 400
/// a plain missing cookie would give, because an absent session is "you are
/// not signed in", not "your request was malformed".
final class SessionIdExtractable implements FromRequestParts<String> {
  /// Reads the session id from the cookie named [name].
  const SessionIdExtractable({this.name = 'session'});

  /// The cookie holding the session id.
  final String name;

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    final value = CookieJar.of(request)[name];
    if (value == null || value.isEmpty) {
      return Err(
        Rejection.unauthorized(
          'expected a $name cookie',
          challenge: 'Cookie',
        ),
      );
    }
    return Ok(value);
  }
}

/// Tries each extractor in order and takes the first that succeeds.
///
/// How one endpoint serves a browser session and a machine key at once:
///
/// ```dart
/// const signedIn = FirstOf([SessionScheme(), ApiKeyScheme()]);
/// ```
///
/// When every scheme declines for want of a credential, the refusal names
/// **all** of them: HTTP allows several challenges in one response, and a 401
/// offering only the last scheme tried tells a browser to do the wrong thing.
///
/// ```http
/// WWW-Authenticate: Bearer, Basic realm="api", Cookie
/// ```
///
/// A challenge carrying its own parameters — `Basic realm="api",
/// charset="UTF-8"` — already contains a comma, and the joined header stays
/// unambiguous anyway: an auth-param contains `=` and a scheme name does not,
/// so a parser attaches the parameter to the scheme before it.
///
/// A refusal that is **not** a 401 wins over that, because it is more specific:
/// a 403 means a credential was presented and was not good enough, and burying
/// it under "no credentials" sends the caller looking for the wrong problem.
///
/// A 5xx from any extractor stops the search immediately: that means something
/// on the server is broken, and trying the next scheme would turn a fault into
/// a confusing 401.
final class FirstOf<T> implements FromRequestParts<T> {
  /// Accepts whichever of [extractors] succeeds first.
  const FirstOf(this.extractors);

  /// The extractors to try, in order.
  final List<FromRequestParts<T>> extractors;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    final refusals = <Rejection>[];

    for (final extractor in extractors) {
      switch (await extractor.extract(request)) {
        case Ok(:final value):
          return Ok(value);
        case Err(:final error) when error.status >= 500:
          return Err(error);
        case Err(:final error):
          refusals.add(error);
      }
    }

    if (refusals.isEmpty) {
      return const Err(Rejection.unauthorized('no credentials accepted'));
    }

    // Anything other than a 401 says a credential *was* presented and was not
    // good enough. That is the more useful answer, so it is not buried under a
    // challenge asking for one.
    final refused = refusals.where((rejection) => rejection.status != 401);
    if (refused.isNotEmpty) return Err(refused.last);

    // Every scheme wanted a credential and none arrived, so offer them all.
    final challenges = <String>{
      for (final rejection in refusals)
        if (rejection.challenge case final challenge?) challenge,
    };

    return Err(
      Rejection.unauthorized(
        'no credentials were supplied',
        challenge: challenges.isEmpty ? 'Bearer' : challenges.join(', '),
      ),
    );
  }
}

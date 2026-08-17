/// Shortcuts for the built-in extractors.
///
/// The classes stay the vocabulary a generator emits, where a long explicit
/// name is a feature. Written by hand they are noise, and a handler assembled
/// from `path<String>('id')` and `state<Repo>()` reads like the axum signature
/// it stands for.
///
/// These names are short enough to collide with an application's own — `path`
/// with `package:path` in particular. Import with a prefix when that happens;
/// the classes underneath are unaffected.
library;

import 'dart:typed_data';

import 'package:dust_dart/derive.dart';
import 'package:shelf/shelf.dart';

import '../response/rejection.dart';
import 'authorization.dart';
import 'context.dart';
import 'cookie.dart';
import 'credentials.dart';
import 'extractable.dart';
import 'form.dart';
import 'header.dart';
import 'host.dart';
import 'json.dart';
import 'multipart.dart';
import 'path.dart';
import 'query.dart';
import 'raw.dart';
import 'state.dart';
import 'validated.dart';

/// Reads the `{name}` path segment, coerced to [T].
FromRequestParts<T> path<T>(String name) => PathExtractable<T>(name);

/// Reads one query value, coerced to [T]. A nullable [T] makes it optional.
FromRequestParts<T> query<T>(String name) => QueryExtractable<T>(name);

/// Reads every value of a repeated query key, coerced to [T].
FromRequestParts<List<T>> queryList<T>(String name) =>
    QueryListExtractable<T>(name);

/// Reads the application state of type [T] attached with `withState`.
FromRequestParts<T> state<T extends Object>() => StateExtractable<T>();

/// Reads one header, or `null` when it is absent.
FromRequestParts<String?> header(String name) => HeaderExtractable(name);

/// Reads the `Host` header, which the client controls — compare, do not trust.
FromRequestParts<String> host() => const HostExtractable();

/// Reads every header at once.
FromRequestParts<Map<String, String>> headers() => const HeaderMapExtractable();

/// Reads every query pair at once.
FromRequestParts<Map<String, String>> queries() =>
    const QueriesExtractable(_identity);

/// Reads the raw, undecoded query string, or `null` when there is none.
FromRequestParts<String?> rawQuery() => const RawQueryExtractable();

/// Reads one cookie, coerced to [T]. A nullable [T] makes it optional.
FromRequestParts<T> cookie<T>(String name) => CookieExtractable<T>(name);

/// Reads every cookie at once.
FromRequestParts<CookieJar> cookies() => const CookieJarExtractable();

/// Reads the token from an `Authorization: Bearer` header.
FromRequestParts<String> bearerToken() => const BearerTokenExtractable();

/// Reads and decodes an `Authorization: Basic` header.
FromRequestParts<BasicCredentials> basicCredentials(
        {String realm = 'restricted'}) =>
    BasicCredentialsExtractable(realm: realm);

/// Reads an API key from a header, falling back to the query string.
FromRequestParts<String> apiKey({
  String header = 'x-api-key',
  String query = 'api_key',
  bool allowQuery = true,
}) =>
    ApiKeyExtractable(header: header, query: query, allowQuery: allowQuery);

/// Reads a session identifier from a cookie.
FromRequestParts<String> sessionId({String name = 'session'}) =>
    SessionIdExtractable(name: name);

/// Takes whichever of [extractors] succeeds first.
FromRequestParts<T> firstOf<T>(List<FromRequestParts<T>> extractors) =>
    FirstOf<T>(extractors);

/// Decodes a JSON object body with [deserialize].
FromRequest<T> body<T>(T Function(Map<String, Object?> json) deserialize) =>
    JsonExtractable<T>(deserialize);

/// Decodes a JSON array body with [deserialize].
FromRequest<List<T>> bodyList<T>(
  T Function(Map<String, Object?> json) deserialize,
) =>
    JsonListExtractable<T>(deserialize);

/// Reads the body as UTF-8 text.
FromRequest<String> textBody() => const TextBodyExtractable();

/// Reads the body as raw bytes.
FromRequest<Uint8List> rawBody() => const RawBodyExtractable();

/// Hands the body to the handler as an unread stream.
FromRequest<Stream<List<int>>> bodyStream() => const StreamBodyExtractable();

/// Decodes an `application/x-www-form-urlencoded` body.
FromRequest<FormMap> form() => const FormExtractable();

/// Decodes a `multipart/form-data` body.
FromRequest<MultipartForm> multipart() => const MultipartExtractable();

/// The connection's peer address and port.
FromRequestParts<PeerInfo> peer() => const PeerExtractable();

/// The request itself, for a handler that wants to reach past the extractors.
///
/// Named for what it hands over rather than just `request`, which collides
/// with the local variable almost every test and middleware already has.
FromRequestParts<Request> rawRequest() => const _RequestExtractable();

/// Runs the generated `validate` after [inner] builds the value.
FromRequestParts<T> valid<T extends Validatable>(FromRequestParts<T> inner) =>
    ValidatedExtractable<T>(inner);

/// Turns a client-error rejection from [inner] into `None`.
FromRequestParts<Option<T>> optional<T>(FromRequestParts<T> inner) =>
    OptionalExtractable<T>(inner);

/// Hands [inner]'s rejection to the handler instead of short-circuiting.
FromRequestParts<Result<T, Rejection>> fallible<T>(
  FromRequestParts<T> inner,
) =>
    FallibleExtractable<T>(inner);

Map<String, String> _identity(Map<String, String> queries) => queries;

final class _RequestExtractable implements FromRequestParts<Request> {
  const _RequestExtractable();

  @override
  Future<Result<Request, Rejection>> extract(Request request) async =>
      Ok(request);
}

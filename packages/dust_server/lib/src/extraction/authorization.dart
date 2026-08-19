import 'dart:convert';

import 'package:dust_dart/fp.dart';
import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../response/rejection.dart';
import 'extractable.dart';

/// An `Authorization` header, split where the specification splits it.
///
/// The header is `<scheme> <credentials>`, and the scheme is compared
/// case-insensitively — a client sending `bearer` rather than `Bearer` is
/// conforming, and an extractor that misses it is not.
final class Authorization {
  /// Pairs a [scheme] with its [credentials].
  const Authorization(this.scheme, this.credentials);

  /// Reads the header of [request], or `null` when there is none.
  static Authorization? of(Request request) {
    final raw = RequestParts.of(request).headers['authorization'];
    if (raw == null) return null;

    final separator = raw.indexOf(' ');
    if (separator < 1) return Authorization(raw.toLowerCase(), '');
    return Authorization(
      raw.substring(0, separator).toLowerCase(),
      raw.substring(separator + 1).trim(),
    );
  }

  /// The scheme, lower-cased: `bearer`, `basic`, `digest`.
  final String scheme;

  /// Everything after the scheme.
  final String credentials;

  /// Whether this header uses [name], which is matched case-insensitively.
  bool isScheme(String name) => scheme == name.toLowerCase();
}

/// Extracts the token from an `Authorization: Bearer <token>` header.
///
/// Every bearer scheme starts by stripping that prefix and answering 401 with
/// the right challenge when it is missing. Doing it once means a custom
/// extractor starts from the token rather than from the header.
final class BearerTokenExtractable implements FromRequestParts<String> {
  /// Reads a bearer token.
  const BearerTokenExtractable({this.challenge = 'Bearer'});

  /// The `WWW-Authenticate` challenge sent with a 401.
  final String challenge;

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    final header = Authorization.of(request);
    if (header == null || !header.isScheme('bearer')) {
      return Err(
        Rejection.unauthorized(
          'expected a bearer token',
          challenge: challenge,
        ),
      );
    }
    if (header.credentials.isEmpty) {
      return Err(
        Rejection.unauthorized('the bearer token is empty',
            challenge: challenge),
      );
    }
    return Ok(header.credentials);
  }
}

/// A username and password from an `Authorization: Basic` header.
final class BasicCredentials {
  /// Pairs a [username] with its [password].
  const BasicCredentials(this.username, this.password);

  /// The username.
  final String username;

  /// The password, which may itself contain colons.
  final String password;
}

/// Extracts and decodes an `Authorization: Basic base64(user:password)` header.
///
/// Rejects with the `Basic` challenge rather than `Bearer`, which is what makes
/// a browser show its own password prompt; answering the wrong one is why a 401
/// sometimes does nothing visible.
final class BasicCredentialsExtractable
    implements FromRequestParts<BasicCredentials> {
  /// Reads basic credentials for [realm].
  const BasicCredentialsExtractable({this.realm = 'restricted'});

  /// The realm a browser shows in its prompt.
  final String realm;

  Rejection get _challenge => Rejection.unauthorized(
        'expected basic credentials',
        challenge: 'Basic realm="$realm", charset="UTF-8"',
      );

  @override
  Future<Result<BasicCredentials, Rejection>> extract(Request request) async {
    final header = Authorization.of(request);
    if (header == null || !header.isScheme('basic')) return Err(_challenge);

    final String decoded;
    try {
      decoded = utf8.decode(base64.decode(header.credentials));
    } on FormatException {
      return Err(_challenge);
    }

    // Only the first colon separates; a password may contain more.
    final separator = decoded.indexOf(':');
    if (separator < 1) return Err(_challenge);

    return Ok(
      BasicCredentials(
        decoded.substring(0, separator),
        decoded.substring(separator + 1),
      ),
    );
  }
}

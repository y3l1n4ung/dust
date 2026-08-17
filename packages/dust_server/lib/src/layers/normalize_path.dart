import 'package:shelf/shelf.dart';

import '../response/redirect.dart';
import '../router/middleware.dart';

/// What to do about a trailing slash.
enum TrailingSlash {
  /// Strip it before matching: `/todos/` is served by `/todos`.
  strip,

  /// Add it before matching: `/todos` is served by `/todos/`.
  append,
}

/// Makes `/todos` and `/todos/` mean the same thing.
///
/// The router treats them as different paths on purpose — a route table that
/// silently matched both would hide a duplicate. This layer decides the
/// question once, at the edge, instead of every route declaring itself twice.
///
/// ```dart
/// final app = Router()
///   ..layer(const NormalizePath())
///   ..merge(routes);
/// ```
///
/// Two ways to settle it, and the choice is not cosmetic:
///
/// * **Rewrite** (the default) serves the normalized path directly. One
///   request, one response, and the URL in the address bar is left alone.
/// * **Redirect** answers 308 and sends the client to the canonical form. It
///   costs a round trip and it collapses two URLs into one for caches, logs,
///   and search engines — which is the reason to pay for it.
///
/// The root path is never touched: `/` is already canonical, and stripping its
/// slash would leave an empty path nothing can match.
final class NormalizePath implements Layer {
  /// Normalizes by rewriting the request.
  const NormalizePath({this.slash = TrailingSlash.strip}) : redirect = false;

  /// Normalizes by redirecting the client to the canonical form.
  ///
  /// A 308 keeps the method, so a `POST` to `/todos/` arrives at `/todos` as a
  /// `POST` rather than being turned into a `GET`.
  const NormalizePath.redirecting({this.slash = TrailingSlash.strip})
      : redirect = true;

  /// Which form is canonical.
  final TrailingSlash slash;

  /// Whether to redirect rather than rewrite.
  final bool redirect;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final path = request.url.path;
        final normalized = _normalize(path);
        if (normalized == null) return inner(request);

        if (redirect) {
          final target = request.requestedUri.replace(path: '/$normalized');
          return Redirect.permanent(
            '${target.path}'
            '${target.hasQuery ? '?${target.query}' : ''}',
          ).intoResponse();
        }

        // `change(path:)` strips a prefix rather than rewriting the URL, so
        // the request is rebuilt at the normalized path. The body is handed on
        // unread: normalizing a path must not consume it.
        return inner(
          Request(
            request.method,
            request.requestedUri.replace(path: '/$normalized'),
            headers: request.headers,
            body: request.read(),
            context: request.context,
            protocolVersion: request.protocolVersion,
          ),
        );
      };
    };
  }

  /// The canonical form of [path], or `null` when it already is canonical.
  String? _normalize(String path) {
    // `/` has no trailing slash to strip and nothing to append to.
    if (path.isEmpty) return null;

    switch (slash) {
      case TrailingSlash.strip:
        if (!path.endsWith('/')) return null;
        final stripped = path.substring(0, path.length - 1);
        return stripped.isEmpty ? null : stripped;
      case TrailingSlash.append:
        return path.endsWith('/') ? null : '$path/';
    }
  }
}

/// Adds the response headers a browser uses to lock a page down.
///
/// None of these are a substitute for the server being correct; each one turns
/// a class of mistake into a smaller one.
///
/// ```dart
/// final app = Router()
///   ..layer(const SecurityHeaders())
///   ..merge(routes);
/// ```
///
/// Every value is replaceable, and a `null` drops the header entirely — a
/// default that cannot be turned off is a default that gets forked.
final class SecurityHeaders implements Layer {
  /// Adds the usual set.
  const SecurityHeaders({
    this.contentTypeOptions = 'nosniff',
    this.frameOptions = 'DENY',
    this.referrerPolicy = 'strict-origin-when-cross-origin',
    this.contentSecurityPolicy,
    this.strictTransportSecurity,
  });

  /// Stops a browser guessing a type other than the one you sent.
  ///
  /// Without it, a text file a user uploaded can be sniffed as HTML and run.
  final String? contentTypeOptions;

  /// Stops the page being framed, which is what clickjacking needs.
  final String? frameOptions;

  /// How much of the URL to leak when following a link off-site.
  final String? referrerPolicy;

  /// A content security policy, when the application has one.
  ///
  /// No default: a wrong CSP breaks a working page, and a permissive one is
  /// theatre. It belongs to the application that knows what it loads.
  final String? contentSecurityPolicy;

  /// Tells a browser to use HTTPS for this host from now on.
  ///
  /// No default either: sent over plain HTTP it does nothing, and sent from a
  /// host whose certificate later lapses it locks users out of the site.
  final String? strictTransportSecurity;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final response = await inner(request);
        return response.change(
          headers: {
            if (contentTypeOptions case final value?)
              'x-content-type-options': value,
            if (frameOptions case final value?) 'x-frame-options': value,
            if (referrerPolicy case final value?) 'referrer-policy': value,
            if (contentSecurityPolicy case final value?)
              'content-security-policy': value,
            if (strictTransportSecurity case final value?)
              'strict-transport-security': value,
          },
        );
      };
    };
  }
}

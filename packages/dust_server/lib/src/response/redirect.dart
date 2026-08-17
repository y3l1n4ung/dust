import 'package:shelf/shelf.dart';

import 'into_response.dart';

/// Sends the client somewhere else.
///
/// Return one from a handler; the verb builder writes it like any other value:
///
/// ```dart
/// Future<Object?> oldPath(Request request) async =>
///     Redirect.permanent('/new/path');
/// ```
///
/// Which constructor to reach for is a question about **method and caching**,
/// and getting it wrong is how a `POST` turns into a `GET` or a browser
/// remembers a move that was meant to be temporary:
///
/// | Constructor | Status | Method | Cached |
/// | :--- | :--- | :--- | :--- |
/// | [Redirect.to] | 303 | always becomes `GET` | no |
/// | [Redirect.temporary] | 307 | preserved | no |
/// | [Redirect.permanent] | 308 | preserved | **forever** |
/// | [Redirect.found] | 302 | *usually* becomes `GET` | no |
/// | [Redirect.movedPermanently] | 301 | *usually* becomes `GET` | **forever** |
///
/// After a successful `POST`, [Redirect.to] is almost always the one you want:
/// 303 tells the browser to follow up with a `GET`, which is what stops a
/// refresh from submitting the form twice.
///
/// 301 and 302 are kept because real clients send them, but they are the two
/// whose method handling was never specified consistently — browsers turn a
/// `POST` into a `GET` and other clients do not. Prefer 303 or 307, which say
/// what they mean.
final class Redirect implements IntoResponse {
  const Redirect._(this.status, this.location);

  /// A 303: follow up with a `GET`, whatever this request was.
  ///
  /// The right answer after a form post.
  const Redirect.to(String location) : this._(303, location);

  /// A 307: same place, same method, do not cache.
  const Redirect.temporary(String location) : this._(307, location);

  /// A 308: same method, and remember this **permanently**.
  ///
  /// A browser that caches it may never ask again, so a permanent redirect
  /// pointing somewhere wrong is expensive to take back.
  const Redirect.permanent(String location) : this._(308, location);

  /// A 302, kept for clients that expect it. Prefer [Redirect.to].
  const Redirect.found(String location) : this._(302, location);

  /// A 301, kept for clients that expect it. Prefer [Redirect.permanent].
  const Redirect.movedPermanently(String location) : this._(301, location);

  /// The status code this redirect sends.
  final int status;

  /// Where the client is being sent.
  final String location;

  /// Whether a client may cache this redirect indefinitely.
  bool get isPermanent => status == 301 || status == 308;

  /// Whether the client keeps the method it used.
  bool get preservesMethod => status == 307 || status == 308;

  @override
  Response intoResponse() {
    return Response(
      status,
      // A newline in `Location` would end the header and start another, and a
      // redirect target is exactly the kind of value built from user input.
      headers: {'location': location.replaceAll(RegExp(r'[\r\n\x00]'), '')},
    );
  }

  @override
  String toString() => 'Redirect($status, $location)';
}

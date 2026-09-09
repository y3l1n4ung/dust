import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;

/// Helpers shared by the example suites.
///
/// `_valueOf` was private while every example lived in one file. It is
/// `valueOf` here because the suites that read it are now separate libraries.

/// The `seen` count out of a `/whoami` body.
int app0(http.Response response) =>
    jsonDecode(response.body)['seen'] as int? ?? -1;

/// The cookie value out of a `Set-Cookie` header.
String valueOf(String setCookie) =>
    setCookie.split(';').first.split('=').sublist(1).join('=');

/// Keeps every span so a test can look at one.
final class CollectingExporter implements SpanExporter {
  /// What has been exported.
  final spans = <Span>[];

  @override
  void export(Span span) => spans.add(span);
}

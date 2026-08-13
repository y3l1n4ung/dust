import 'package:dust_flutter/route.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('parses platform route information with query and fragment', () async {
    final parser = GeneratedRouteInformationParser<_Route>(
      parseRoute: _Route.fromUri,
      routeLocation: (route) => route.location,
    );

    final route = await parser.parseRouteInformation(
      RouteInformation(
        uri: Uri.parse('/projects/42?tab=activity&archived=true#comments'),
      ),
    );

    expect(route.path, '/projects/42');
    expect(route.queryParameters, {
      'tab': 'activity',
      'archived': 'true',
    });
    expect(route.fragment, 'comments');
  });

  test('restores typed route information from generated locations', () {
    final parser = GeneratedRouteInformationParser<_Route>(
      parseRoute: _Route.fromUri,
      routeLocation: (route) => route.location,
    );

    final restored = parser.restoreRouteInformation(
      const _Route('/projects/42?tab=files#attachments'),
    );

    expect(restored.uri.toString(), '/projects/42?tab=files#attachments');
  });
}

final class _Route {
  const _Route(this.location);

  factory _Route.fromUri(Uri uri) => _Route(uri.toString());

  final String location;

  Uri get uri => Uri.parse(location);

  String get path => uri.path;

  Map<String, String> get queryParameters => uri.queryParameters;

  String get fragment => uri.fragment;
}

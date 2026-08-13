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

  test('passes absolute platform URLs to the generated parser', () async {
    final parser = GeneratedRouteInformationParser<_Route>(
      parseRoute: _Route.fromUri,
      routeLocation: (route) => route.location,
    );

    final webRoute = await parser.parseRouteInformation(
      RouteInformation(
        uri: Uri.parse(
          'https://shop.example/products/42?tab=activity#comments',
        ),
      ),
    );
    final appRoute = await parser.parseRouteInformation(
      RouteInformation(
        uri: Uri.parse(
          'shopping://open/orders/ORDER%201%2F2?campaign=spring#receipt',
        ),
      ),
    );

    expect(webRoute.scheme, 'https');
    expect(webRoute.host, 'shop.example');
    expect(webRoute.path, '/products/42');
    expect(webRoute.queryParameters, {'tab': 'activity'});
    expect(webRoute.fragment, 'comments');
    expect(appRoute.scheme, 'shopping');
    expect(appRoute.host, 'open');
    expect(appRoute.path, '/orders/ORDER%201%2F2');
    expect(appRoute.pathSegments, ['orders', 'ORDER 1/2']);
    expect(appRoute.queryParameters, {'campaign': 'spring'});
    expect(appRoute.fragment, 'receipt');
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

  List<String> get pathSegments => uri.pathSegments;

  String get scheme => uri.scheme;

  String get host => uri.host;

  Map<String, String> get queryParameters => uri.queryParameters;

  String get fragment => uri.fragment;
}

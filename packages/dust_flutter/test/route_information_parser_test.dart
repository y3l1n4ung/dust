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

  test('runs router route-information override before generated parsing',
      () async {
    final router = _PrefixRouter();
    final state = Object();
    final parser = GeneratedRouteInformationParser<_Route>(
      router: router,
      parseRoute: (uri) => _Route(_appLocation(uri)),
      routeLocation: (route) => route.location,
    );

    final prefixedRoute = await parser.parseRouteInformation(
      RouteInformation(
        uri: Uri.parse(
          'https://shop.example/app/projects/42?tab=activity#comments',
        ),
        state: state,
      ),
    );
    final unsafeHostRoute = await parser.parseRouteInformation(
      RouteInformation(
        uri: Uri.parse('https://evil.test/app/projects/42'),
        state: state,
      ),
    );

    expect(prefixedRoute.location, '/projects/42?tab=activity#comments');
    expect(
      unsafeHostRoute.location,
      '/404?path=https%3A%2F%2Fevil.test%2Fapp%2Fprojects%2F42',
    );
    expect(router.seenStates.length, 2);
    expect(router.seenStates.every((seen) => identical(seen, state)), isTrue);
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

String _appLocation(Uri uri) {
  final query = uri.hasQuery ? '?${uri.query}' : '';
  final fragment = uri.fragment.isEmpty ? '' : '#${uri.fragment}';
  return '${uri.path}$query$fragment';
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

final class _PrefixRouter extends RouterBase<_Route> {
  final seenStates = <Object?>[];

  @override
  RouteInformation parseRouteInformation(RouteInformation information) {
    seenStates.add(information.state);
    final uri = information.uri;
    if (uri.hasAuthority && uri.host != 'shop.example') {
      return RouteInformation(
        uri: Uri(path: '/404', queryParameters: {'path': uri.toString()}),
        state: information.state,
      );
    }
    if (!uri.path.startsWith('/app/')) return information;
    return RouteInformation(
      uri: uri.replace(path: uri.path.substring('/app'.length)),
      state: information.state,
    );
  }
}

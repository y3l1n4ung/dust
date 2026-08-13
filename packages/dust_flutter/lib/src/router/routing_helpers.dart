part of 'routing_core.dart';

/// URI query and fragment values preserved from an incoming route.
///
/// This is exported for generated route code. App code should not construct it.
final class GeneratedRouteUriExtras {
  /// Creates preserved route URI extras.
  const GeneratedRouteUriExtras({
    required this.queryParameters,
    required this.fragment,
  });

  /// Query parameters not represented by typed route fields.
  final Map<String, List<String>> queryParameters;

  /// URI fragment from the incoming route.
  final String fragment;
}

final Expando<GeneratedRouteUriExtras> _generatedRouteUriExtras =
    Expando<GeneratedRouteUriExtras>();

/// Returns preserved URI extras for a generated route instance.
GeneratedRouteUriExtras? generatedRouteUriExtrasOf(Object route) {
  return _generatedRouteUriExtras[route];
}

/// Stores unknown query parameters and fragments on a parsed generated route.
T withGeneratedRouteUriExtras<T extends Object>(
  T route,
  Uri uri,
  Set<String> knownQueryParameters,
) {
  final queryParameters = <String, List<String>>{};
  for (final entry in uri.queryParametersAll.entries) {
    if (!knownQueryParameters.contains(entry.key)) {
      queryParameters[entry.key] = List.unmodifiable(entry.value);
    }
  }
  if (queryParameters.isEmpty && uri.fragment.isEmpty) return route;
  _generatedRouteUriExtras[route] = GeneratedRouteUriExtras(
    queryParameters: Map.unmodifiable(queryParameters),
    fragment: uri.fragment,
  );
  return route;
}

/// Builds a generated route location from path segments, query, and extras.
String generatedRoutePath(
  List<String> segments, {
  Map<String, dynamic>? queryParameters,
  GeneratedRouteUriExtras? uriExtras,
}) {
  final query = <String, dynamic>{
    ...?queryParameters,
    ...?uriExtras?.queryParameters,
  };
  final text = Uri(
    pathSegments: segments,
    queryParameters: query.isEmpty ? null : query,
    fragment: uriExtras?.fragment.isEmpty ?? true ? null : uriExtras!.fragment,
  ).toString();
  if (text.isEmpty) return '/';
  return text.startsWith('/') ? text : '/$text';
}

/// Parses the URL bool spelling supported by generated route parameters.
bool? generatedRouteParseBool(String? value) {
  return switch (value) {
    'true' || '1' => true,
    'false' || '0' => false,
    null || '' => null,
    _ => null,
  };
}

/// Parses a URL enum value by Dart enum name.
T? generatedRouteParseEnum<T extends Enum>(Iterable<T> values, String? value) {
  if (value == null || value.isEmpty) return null;
  for (final item in values) {
    if (item.name == value) return item;
  }
  return null;
}

/// Parses repeated URL query values as integers.
List<int>? generatedRouteParseIntList(List<String>? values) {
  if (values == null) return null;
  final parsed = <int>[];
  for (final value in values) {
    final item = int.tryParse(value);
    if (item == null) return null;
    parsed.add(item);
  }
  return List.unmodifiable(parsed);
}

/// Compares generated route list defaults without relying on identity.
bool generatedRouteListEquals<T>(List<T> left, List<T> right) {
  if (identical(left, right)) return true;
  if (left.length != right.length) return false;
  for (var index = 0; index < left.length; index += 1) {
    if (left[index] != right[index]) return false;
  }
  return true;
}

/// Checks that generated page wrapping matches generated route metadata.
bool generatedRouteShellsMatch(
  List<GeneratedRoute> routes,
  Map<Type, Type?> appliedShellsByPage,
) {
  bool visit(GeneratedRoute route) {
    final page = route.page;
    if (page != null && appliedShellsByPage[page] != route.shell) {
      return false;
    }
    return route.routes.every(visit);
  }

  return routes.every(visit);
}

/// Page transition builder used by generated no-transition routes.
final class GeneratedNoTransitionBuilder extends PageTransitionsBuilder {
  /// Creates a no-transition page transition builder.
  const GeneratedNoTransitionBuilder();

  @override
  Duration get transitionDuration => Duration.zero;

  @override
  Duration get reverseTransitionDuration => Duration.zero;

  @override
  Widget buildTransitions<T>(
    PageRoute<T> route,
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    return child;
  }
}

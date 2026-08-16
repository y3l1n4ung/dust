/// One route as it is actually served, for anything that inspects the app.
///
/// This is the whole introspection contract. A package that generates API
/// documentation reads the method and path from here and finds its own
/// description in [metadata], which keeps `dust_server` free of any particular
/// documentation format.
final class MountedRoute {
  /// Describes one served route.
  const MountedRoute({
    required this.method,
    required this.path,
    this.metadata = const [],
  });

  /// The uppercase HTTP method.
  final String method;

  /// The full path, with `{name}` placeholders intact.
  final String path;

  /// Metadata from every group this route was mounted through, outermost
  /// first, with `null` entries left out.
  final List<Object> metadata;

  /// The first piece of metadata of type [T], searching innermost first.
  ///
  /// Group metadata is usually nested, and the closest one wins.
  T? metadataOf<T>() {
    for (final entry in metadata.reversed) {
      if (entry is T) return entry as T;
    }
    return null;
  }

  @override
  String toString() => 'MountedRoute($method $path)';
}

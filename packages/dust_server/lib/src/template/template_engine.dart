/// Renders a named template against a set of values.
///
/// `dust_server` does not implement templating. It ships an adapter over
/// `mustache_template`, which is the most used template package on pub, and
/// this interface so anything else, `jinja` and `liquify` included, can be
/// dropped in without changing a handler.
///
/// ```dart
/// final class JinjaTemplates implements TemplateEngine {
///   @override
///   String render(String name, Map<String, Object?> values) => ...;
/// }
/// ```
abstract interface class TemplateEngine {
  /// Renders the template called [name] with [values].
  ///
  /// Throws [TemplateNotFound] when no such template exists, so a typo is a
  /// failure rather than an empty page.
  String render(String name, Map<String, Object?> values);
}

/// A template nobody registered.
final class TemplateNotFound implements Exception {
  /// Reports that [name] is unknown, listing what is [available].
  const TemplateNotFound(this.name, this.available);

  /// The template that was asked for.
  final String name;

  /// The templates that do exist, sorted.
  final List<String> available;

  @override
  String toString() {
    final known =
        available.isEmpty ? 'none are registered' : available.join(', ');
    return 'TemplateNotFound: no template named "$name" ($known)';
  }
}

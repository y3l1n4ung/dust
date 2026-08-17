import 'dart:io';

import 'package:mustache_template/mustache_template.dart' as mustache;

import 'template_engine.dart';

/// A [TemplateEngine] over `mustache_template`.
///
/// Templates are parsed once and reused, and partials resolve against the same
/// set, so a layout including a header costs one parse each.
///
/// ```dart
/// final templates = MustacheTemplates.fromDirectory('web/templates');
/// final app = Router()..withState(templates);
/// ```
final class MustacheTemplates implements TemplateEngine {
  /// Compiles every entry of [sources], keyed by template name.
  MustacheTemplates(Map<String, String> sources) {
    for (final entry in sources.entries) {
      _templates[entry.key] = mustache.Template(
        entry.value,
        name: entry.key,
        partialResolver: _partial,
        htmlEscapeValues: true,
        lenient: false,
      );
    }
  }

  /// Loads every file under [directory] whose name ends with [extension].
  ///
  /// A template's name is its path below [directory] without the extension, so
  /// `web/templates/mail/welcome.html` is `mail/welcome`.
  ///
  /// That name works for [render]. It does not work in a `{{> partial}}` tag:
  /// `mustache_template` allows only letters, digits, `-`, `_` and `.` there,
  /// so a fragment meant to be included has to sit at the root or carry a flat
  /// name. A slash in a partial tag fails at load time rather than silently
  /// rendering nothing.
  factory MustacheTemplates.fromDirectory(
    String directory, {
    String extension = '.html',
  }) {
    final root = Directory(directory);
    if (!root.existsSync()) {
      throw ArgumentError('no template directory at "$directory"');
    }

    final sources = <String, String>{};
    for (final entry in root.listSync(recursive: true).whereType<File>()) {
      if (!entry.path.endsWith(extension)) continue;

      final relative = entry.path
          .substring(root.path.length)
          .replaceAll(r'\', '/')
          .replaceFirst(RegExp('^/'), '');
      final name = relative.substring(0, relative.length - extension.length);
      sources[name] = entry.readAsStringSync();
    }
    return MustacheTemplates(sources);
  }

  final _templates = <String, mustache.Template>{};

  /// The names that can be rendered, sorted.
  List<String> get names => _templates.keys.toList()..sort();

  @override
  String render(String name, Map<String, Object?> values) {
    final template = _templates[name];
    if (template == null) throw TemplateNotFound(name, names);
    return template.renderString(values);
  }

  mustache.Template? _partial(String name) => _templates[name];
}

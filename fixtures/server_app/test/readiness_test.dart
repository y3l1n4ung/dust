import 'dart:io';

import 'package:test/test.dart';

/// Checks the fixture is what it claims to be, by reading its own source.
///
/// The fixture's whole value is being the spec a plugin is built against, and
/// a spec that contradicts itself is worse than none. Nothing else here catches
/// that: an annotated handler that is never routed compiles cleanly and passes
/// every other test in this package, because no request is ever sent to a route
/// that does not exist.
///
/// These are the rules a generator would have to follow, checked against the
/// hand-written output that stands in for it.

final List<Directory> _features = Directory('lib/src/features')
    .listSync()
    .whereType<Directory>()
    .toList()
  ..sort((a, b) => a.path.compareTo(b.path));

String _name(Directory feature) =>
    feature.path.split(Platform.pathSeparator).last;

File _handler(Directory feature) =>
    File('${feature.path}/${_name(feature)}_handler.dart');

File _emitted(Directory feature) =>
    File('${feature.path}/${_name(feature)}_handler.g.dart');

/// The module function a file called `<base>.dart` produces.
///
/// The file base, lowerCamelCased, plus `Routes`, prefixed with `$`. No
/// stripping and no English: `accounts_handler.dart` gives
/// `\$accountsHandlerRoutes`, which is verbose and needs nothing remembered.
String _moduleName(String fileBase) {
  final parts = fileBase.split('_');
  final camel = parts.first +
      parts
          .skip(1)
          .map((part) => part[0].toUpperCase() + part.substring(1))
          .join();

  return '\$${camel}Routes';
}

/// Every `@GET('/x')`-style annotation in [source], as `METHOD path`.
List<String> _annotatedRoutes(String source) {
  final pattern = RegExp(
    r"^@(GET|POST|PUT|PATCH|DELETE|HEAD)\('([^']*)'",
    multiLine: true,
  );

  return pattern
      .allMatches(source)
      .map((match) => '${match.group(1)} ${match.group(2)}')
      .toList();
}

/// Every `Route('GET', '/x', ...)` in [emitted], as `METHOD path`.
List<String> _emittedRoutes(String emitted) {
  final pattern = RegExp(r"Route\('([A-Z]+)',\s*'([^']*)'");

  return pattern
      .allMatches(emitted)
      .map((match) => '${match.group(1)} ${match.group(2)}')
      .toList();
}

void main() {
  test('there are features to check', () {
    expect(_features, isNotEmpty);
  });

  for (final feature in _features) {
    final name = _name(feature);

    group('the $name feature', () {
      test('has a handler file named after the feature', () {
        expect(_handler(feature).existsSync(), isTrue,
            reason: '${_handler(feature).path} is missing');
      });

      test('declares its generated part as <source>.g.dart', () {
        expect(
          _handler(feature).readAsStringSync(),
          contains("part '${name}_handler.g.dart';"),
        );
      });

      test('the emitted file is a part of the handler', () {
        expect(
          _emitted(feature).readAsStringSync(),
          contains("part of '${name}_handler.dart';"),
        );
      });

      test('exposes the module name its file name produces', () {
        // The whole rule: file base, lowerCamelCase, Routes, $ prefix. Nothing
        // stripped, nothing pluralised, nothing to remember.
        expect(
          _emitted(feature).readAsStringSync(),
          contains('Router ${_moduleName('${name}_handler')}()'),
        );
      });

      test('routes exactly what the handlers annotate', () {
        // The check that catches drift. Adding an annotated handler and
        // forgetting to route it compiles, analyzes, and passes every other
        // test in this package.
        final annotated =
            _annotatedRoutes(_handler(feature).readAsStringSync());
        final emitted = _emittedRoutes(_emitted(feature).readAsStringSync());

        expect(annotated, isNotEmpty, reason: 'no annotated handlers found');
        expect(
          emitted..sort(),
          equals(annotated..sort()),
          reason: 'the annotations and the route table disagree',
        );
      });

      test('every emitted handler is private and marked generated', () {
        // `_$` is Dust's convention for a generated symbol nobody writes.
        final emitted = _emitted(feature).readAsStringSync();
        final handlers = RegExp(r'Future<Response> ([_$\w]+)\(Request')
            .allMatches(emitted)
            .map((match) => match.group(1)!)
            .toList();

        expect(handlers, isNotEmpty);
        for (final handler in handlers) {
          expect(handler, startsWith(r'_$'), reason: handler);
        }
      });

      test('carries the generated header, so nobody hand-edits it', () {
        expect(
          _emitted(feature).readAsStringSync(),
          startsWith('// GENERATED CODE - DO NOT MODIFY BY HAND'),
        );
      });

      test('emits nothing that needs a private runtime symbol', () {
        // If generated code needs something @internal, the runtime is missing a
        // public affordance — that is a runtime fix, not a wider plugin.
        final emitted = _emitted(feature).readAsStringSync();
        final body = emitted.split("part of '$name.dart';").last;

        expect(
          RegExp(r'\b_(?!\$)[a-z]\w*\(').allMatches(body),
          isEmpty,
          reason: 'generated code reached for a private symbol',
        );
      });
    });
  }
}

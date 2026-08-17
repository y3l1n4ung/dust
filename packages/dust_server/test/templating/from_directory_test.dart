import 'dart:io';

import 'package:dust_server/templating.dart';
import 'package:test/test.dart';

/// Loading templates off disk is the path a deployed application takes, so it
/// is tested against a real directory rather than a map. What matters is the
/// naming rule: a template's name is its path below the root without the
/// extension, because that is what every `render` call has to spell.

void main() {
  late Directory root;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('dust_templates');
  });

  tearDown(() => root.delete(recursive: true));

  void write(String relative, String contents) {
    final file = File('${root.path}/$relative')
      ..parent.createSync(recursive: true);
    file.writeAsStringSync(contents);
  }

  group('MustacheTemplates.fromDirectory', () {
    test('loads a template at the root', () {
      write('index.html', 'hello {{name}}');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(templates.render('index', {'name': 'dust'}), 'hello dust');
    });

    test('names a template by its path below the root', () {
      write('mail/welcome.html', 'welcome');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(templates.names, ['mail/welcome']);
    });

    test('loads templates from more than one level down', () {
      write('mail/en/welcome.html', 'hi');
      write('index.html', 'root');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(templates.names, ['index', 'mail/en/welcome']);
    });

    test('skips files that do not carry the extension', () {
      write('index.html', 'page');
      write('notes.txt', 'not a template');
      write('README.md', 'nor this');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(templates.names, ['index']);
    });

    test('honours a different extension', () {
      write('index.mustache', 'page');
      write('other.html', 'ignored');

      final templates = MustacheTemplates.fromDirectory(
        root.path,
        extension: '.mustache',
      );

      expect(templates.names, ['index']);
    });

    test('resolves a partial against the same directory', () {
      write('layout.html', '<main>{{> body}}</main>');
      write('body.html', 'inner');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(templates.render('layout', const {}), '<main>inner</main>');
    });

    test('renders a nested template by its slash name', () {
      write('parts/footer.html', 'footer');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(templates.render('parts/footer', const {}), 'footer');
    });

    test('refuses a partial tag naming a subdirectory', () {
      // `mustache_template` restricts tag names to letters, digits, `-`, `_`
      // and `.`, so a nested template is renderable by name but cannot be
      // included as a partial. Keep partials at the root, or give the shared
      // fragment a flat name.
      write('page.html', '[{{> parts/footer}}]');
      write('parts/footer.html', 'footer');

      expect(
        () => MustacheTemplates.fromDirectory(root.path),
        throwsA(isA<Exception>()),
      );
    });

    test('escapes interpolated values', () {
      write('index.html', '{{body}}');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(
        templates.render('index', {'body': '<script>'}),
        '&lt;script&gt;',
      );
    });

    test('loads an empty directory without failing', () {
      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(templates.names, isEmpty);
    });

    test('throws when the directory does not exist', () {
      expect(
        () => MustacheTemplates.fromDirectory('${root.path}/missing'),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('names a template that is missing in the failure', () {
      write('index.html', 'page');

      final templates = MustacheTemplates.fromDirectory(root.path);

      expect(
        () => templates.render('absent', const {}),
        throwsA(isA<TemplateNotFound>()),
      );
    });
  });
}

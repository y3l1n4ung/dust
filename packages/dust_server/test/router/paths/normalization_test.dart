import 'package:dust_server/server.dart';
import 'package:test/test.dart';

void main() {
  group('normalizePrefix', () {
    test('collapses empty forms to the empty string', () {
      expect(normalizePrefix(''), '');
      expect(normalizePrefix('/'), '');
      expect(normalizePrefix('//'), '');
      expect(normalizePrefix('  '), '');
    });

    test('adds a leading slash and strips a trailing one', () {
      expect(normalizePrefix('api'), '/api');
      expect(normalizePrefix('/api/'), '/api');
      expect(normalizePrefix('api/v1/'), '/api/v1');
    });

    test('collapses repeated slashes', () {
      expect(normalizePrefix('//api//v1'), '/api/v1');
    });
  });

  group('joinPaths', () {
    test('joins normalized segments', () {
      expect(joinPaths('/api', '/todos'), '/api/todos');
      expect(joinPaths('api/', '/todos/'), '/api/todos');
    });

    test('returns the root for two empty segments', () {
      expect(joinPaths('', ''), '/');
      expect(joinPaths('/', '/'), '/');
    });

    test('keeps a single segment rooted', () {
      expect(joinPaths('', '/todos'), '/todos');
      expect(joinPaths('/todos', ''), '/todos');
    });
  });
}

import 'dart:convert';

import 'package:dust_dart/db.dart';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';

void main() {
  group('Rejection.fromSqlxError', () {
    late List<Object> reported;

    setUp(() {
      reported = <Object>[];
      ServerErrors.reporter = (error, stack) => reported.add(error);
      addTearDown(() => ServerErrors.reporter = null);
    });

    test("a missing row is a 404 with the caller's message", () {
      final rejection = Rejection.fromSqlxError(
        SqlxError.noRows('SELECT 1'),
        notFound: 'no such user',
      );

      expect(rejection.status, 404);
      expect(rejection.message, 'no such user');
      expect(reported, isEmpty);
    });

    test("a duplicate value is a 409 with the caller's message", () {
      final rejection = Rejection.fromSqlxError(
        SqlxError.query('dup', kind: SqlxErrorKind.uniqueViolation),
        conflict: 'that email is taken',
      );

      expect(rejection.status, 409);
      expect(rejection.message, 'that email is taken');
      expect(reported, isEmpty);
    });

    test('messages have defaults', () {
      expect(
          Rejection.fromSqlxError(SqlxError.noRows('q')).message, 'Not found');
      expect(
        Rejection.fromSqlxError(
          SqlxError.query('dup', kind: SqlxErrorKind.uniqueViolation),
        ).message,
        'Conflict',
      );
    });

    test('anything else is a 500 that reports the error and hides it',
        () async {
      final failures = <SqlxError>[
        SqlxError.connection('refused'),
        SqlxError.query('fk', kind: SqlxErrorKind.foreignKeyViolation),
        SqlxError.tooManyRows(expected: 1, actual: 2),
        SqlxError.decode('bad column'),
      ];

      for (final error in failures) {
        final rejection = Rejection.fromSqlxError(error);
        final body = jsonDecode(await rejection.intoResponse().readAsString());

        expect(rejection.status, 500, reason: '$error');
        expect(body, {'error': 'Internal server error'}, reason: '$error');
      }
      expect(reported, failures);
    });
  });
}

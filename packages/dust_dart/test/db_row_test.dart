import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

void main() {
  tearDown(RowMapperRegistry.resetForTest);

  test('RowMapperRegistry maps rows through static fromRow references', () {
    registerRowMapper<_StandaloneUser>(_StandaloneUserFromRow.fromRow);

    final user = RowMapperRegistry.map<_StandaloneUser>(
      _MapRow(<String, Object?>{'id': 8, 'is_active': false}),
    );

    expect(user.id, 8);
    expect(user.isActive, isFalse);
    expect(
      RowMapperRegistry.map<Row>(_MapRow(const <String, Object?>{})),
      isA<Row>(),
    );
  });

  test('RowMapperRegistry reports missing generated mappers', () {
    expect(
      () => RowMapperRegistry.map<_StandaloneUser>(
        _MapRow(const <String, Object?>{}),
      ),
      throwsA(isA<StateError>()),
    );
  });

  test('fake row covers the pasted Row interface shape', () {
    final row = _MapRow(<String, Object?>{
      'id': 1,
      'name': 'Ada',
      'score': 2,
      'active': 'true',
      'created_at': '2026-01-01T10:00:00+06:30',
      'missing': null,
    });

    expect(row.read<int>('id'), 1);
    expect(row.read<String>('name'), 'Ada');
    expect(row.read<double>('score'), 2.0);
    expect(row.read<num>('score'), 2);
    expect(row.readBool('active'), isTrue);
    expect(row.readDateTime('created_at'), DateTime.utc(2026, 1, 1, 3, 30));
    expect(row.readIndex<int>(0), 1);
    expect(row.readIndexNullable<Object?>(99), isNull);
    expect(row.readNullable<Object?>('missing'), isNull);
    expect(() => row.read<int>('missing'), throwsA(isA<SqlxDecodeError>()));
    expect(() => row.read<int>('name'), throwsA(isA<SqlxDecodeError>()));
    expect(() => row.readIndex<String>(0), throwsA(isA<SqlxDecodeError>()));
  });

  test('compatibility aliases delegate to pasted Row names', () {
    final row = _MapRow(<String, Object?>{
      'active': 1,
      'created_at': '2026-01-01T00:00:00Z',
      'value': null,
    });

    expect(row.readOrNull<Object?>('value'), isNull);
    expect(row.readIndexOrNull<Object?>(99), isNull);
    expect(row.readBoolOrNull('active'), isTrue);
    expect(row.readDateTimeOrNull('created_at'), DateTime.utc(2026));
  });
}

final class _StandaloneUser {
  const _StandaloneUser({required this.id, required this.isActive});

  final int id;
  final bool isActive;
}

extension _StandaloneUserFromRow on _StandaloneUser {
  static _StandaloneUser fromRow(Row row) {
    return _StandaloneUser(
      id: row.read<int>('id'),
      isActive: row.readBool('is_active'),
    );
  }
}

final class _MapRow implements Row {
  const _MapRow(this._row);

  final Map<String, Object?> _row;

  @override
  T read<T>(String column) {
    final value = readNullable<T>(column);
    if (value == null) throw SqlxError.nullColumn(column);
    return value;
  }

  @override
  T? readNullable<T>(String column) => _coerce<T>(_row[column], column);

  @override
  T readIndex<T>(int index) {
    final value = readIndexNullable<T>(index);
    if (value == null) throw SqlxError.nullColumn('index $index');
    return value;
  }

  @override
  T? readIndexNullable<T>(int index) {
    if (index < 0 || index >= _row.length) return null;
    return _coerce<T>(_row.values.elementAt(index), 'index $index');
  }

  @override
  bool readBool(String column) {
    final value = readBoolNullable(column);
    if (value == null) throw SqlxError.nullColumn(column);
    return value;
  }

  @override
  bool? readBoolNullable(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is bool) return value;
    if (value is int) return value != 0;
    if (value is String) return value == 'true' || value == '1';
    throw SqlxError.decode('bad bool');
  }

  @override
  DateTime readDateTime(String column) {
    final value = readDateTimeNullable(column);
    if (value == null) throw SqlxError.nullColumn(column);
    return value;
  }

  @override
  DateTime? readDateTimeNullable(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is DateTime) return value.toUtc();
    if (value is String) return DateTime.parse(value).toUtc();
    throw SqlxError.decode('bad DateTime');
  }

  static T? _coerce<T>(Object? value, String column) {
    if (value == null) return null;
    if (T == double && value is int) return value.toDouble() as T;
    if (T == num && value is num) return value as T;
    if (value is T) return value as T;
    throw SqlxError.decode('Column `$column` cannot be read as $T.');
  }
}

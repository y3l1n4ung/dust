import 'package:dust_db_runtime/dust_db_runtime.dart';

void main() {
  const iterations = 200000;
  final sql = r'''
SELECT id, name, '$1 literal', "$2 identifier"
FROM users
WHERE id = $1 OR owner_id = $1 OR email = $2
''';
  final parameters = <Object?>[42, 'ada@example.com'];
  final watch = Stopwatch()..start();
  var bound = 0;
  for (var i = 0; i < iterations; i += 1) {
    bound += rewriteOrdinalPlaceholdersForSqlite(
      sql,
      parameters,
    ).parameters.length;
  }
  watch.stop();
  print(
    'placeholder_rewrite iterations=$iterations bound=$bound elapsed_us=${watch.elapsedMicroseconds}',
  );
}

/// SQL plus parameters after driver-specific placeholder rewriting.
final class PreparedSql {
  const PreparedSql(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;
}

/// Rewrites public `$1`, `$2` placeholders to sqlite `?` placeholders.
///
/// This is intentionally not a validator. Dust validates placeholder shape and
/// parameter counts at build/check time; runtime only performs driver binding.
PreparedSql rewriteOrdinalPlaceholdersForSqlite(
  String sql,
  List<Object?> parameters,
) {
  final out = StringBuffer();
  final ordered = <Object?>[];
  var inSingleQuote = false;
  var inDoubleQuote = false;

  for (var i = 0; i < sql.length; i += 1) {
    final ch = sql[i];
    if (ch == "'" && !inDoubleQuote) {
      out.write(ch);
      if (inSingleQuote && i + 1 < sql.length && sql[i + 1] == "'") {
        i += 1;
        out.write("'");
      } else {
        inSingleQuote = !inSingleQuote;
      }
      continue;
    }
    if (ch == '"' && !inSingleQuote) {
      inDoubleQuote = !inDoubleQuote;
      out.write(ch);
      continue;
    }
    if (ch == r'$' && !inSingleQuote && !inDoubleQuote) {
      final start = i + 1;
      var end = start;
      while (end < sql.length) {
        final code = sql.codeUnitAt(end);
        if (code < 48 || code > 57) break;
        end += 1;
      }
      if (end > start) {
        final index = int.tryParse(sql.substring(start, end));
        if (index != null && index > 0 && index <= parameters.length) {
          ordered.add(parameters[index - 1]);
          out.write('?');
          i = end - 1;
          continue;
        }
      }
    }
    out.write(ch);
  }

  return PreparedSql(out.toString(), ordered);
}

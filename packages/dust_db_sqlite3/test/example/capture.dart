import 'dart:async';

/// Runs one example's `main` and returns the lines it printed.
///
/// The examples are written to be read, so they `print` rather than returning a
/// value a test could assert on. Capturing the zone's output keeps them that
/// way and still lets the suite check what each one claims: an example that
/// compiles but prints the wrong answer is a broken example, and only running
/// it catches that.
Future<List<String>> runExample(Future<void> Function() main) async {
  final lines = <String>[];
  await runZoned(
    main,
    zoneSpecification: ZoneSpecification(
      print: (self, parent, zone, line) => lines.add(line),
    ),
  );
  return lines;
}

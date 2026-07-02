import 'package:dust_dart/derive.dart';

part 'optional_note.g.dart';

/// Optional note model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class OptionalNote with _$OptionalNote {
  /// Creates an [OptionalNote].
  const OptionalNote({required this.id, this.note, this.aliases});

  /// Unique identifier.
  final String id;

  /// Note.
  final String? note;

  /// Aliases.
  final List<String>? aliases;
}

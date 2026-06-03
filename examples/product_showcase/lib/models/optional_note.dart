import 'package:dust_dart/derive.dart';

part 'optional_note.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class OptionalNote with _$OptionalNote {
  const OptionalNote({required this.id, this.note, this.aliases});

  final String id;
  final String? note;
  final List<String>? aliases;
}

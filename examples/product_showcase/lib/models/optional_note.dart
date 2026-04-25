import 'package:derive_annotation/derive_annotation.dart';

part 'optional_note.g.dart';

@Derive([Debug(), Clone(), PartialEq(), Hash(), CopyWith()])
class OptionalNote with _$OptionalNoteDust {
  const OptionalNote({
    required this.id,
    this.note,
    this.aliases,
  });

  final String id;
  final String? note;
  final List<String>? aliases;
}

import 'package:dust_dart/db.dart';
import 'package:dust_dart/serde.dart';

part 'note.g.dart';

/// A note.
///
/// One class, three jobs: it is the row the database returns (`FromRow`), the
/// JSON the API answers with (`Serialize`), and the shape a request is checked
/// against (`Validate`). Splitting those into a row type, a response type, and
/// a request type is a thing to do when they start to differ — not before.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize(), FromRow()])
final class Note with _$Note {
  /// Creates a [Note].
  const Note({required this.id, required this.title, required this.body});

  /// Reads a [Note] from decoded JSON.
  static Note deserialize(Map<String, Object?> json) => _$NoteDeserialize(json);

  /// The primary key.
  final int id;

  /// What the note is called.
  final String title;

  /// What it says.
  final String body;
}

/// What `POST /notes` accepts: a note without its id.
@Derive([ToString(), Eq(), Deserialize(), Validate()])
final class NoteDraft with _$NoteDraft {
  /// Creates a [NoteDraft].
  const NoteDraft({required this.title, this.body = ''});

  /// Reads a [NoteDraft] from decoded JSON.
  static NoteDraft deserialize(Map<String, Object?> json) =>
      _$NoteDraftDeserialize(json);

  /// What the note is called.
  @Validate(
      length: Length(min: 1, max: 120), message: 'must be 1 to 120 characters')
  final String title;

  /// What it says.
  @SerDe(defaultValue: '')
  final String body;
}

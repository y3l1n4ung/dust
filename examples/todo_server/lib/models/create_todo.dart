import 'package:dust_dart/serde.dart';

part 'create_todo.g.dart';

/// The body `POST /api/v1/todos` accepts.
///
/// The `Validate()` derive generates the `validate` that
/// `ValidatedExtractable` calls after decoding. Each `@Validate` below becomes
/// one entry in the 422 body, named by its field, so a client is told which
/// input is wrong rather than being handed one flat message.
@Derive([ToString(), Eq(), Deserialize(), Validate()])
final class CreateTodo with _$CreateTodo {
  /// Creates a [CreateTodo].
  const CreateTodo({
    required this.title,
    required this.assignTo,
    this.done = false,
  });

  /// Reads a [CreateTodo] from decoded JSON.
  static CreateTodo deserialize(Map<String, Object?> json) =>
      _$CreateTodoDeserialize(json);

  /// What to do.
  @Validate(
    length: Length(min: 1, max: 200),
    message: 'must be 1 to 200 characters',
  )
  final String title;

  /// Who to assign it to.
  ///
  /// A caller may assign to themselves always, and to anyone else only with
  /// the `todos:admin` scope. The handler decides that; validation only says
  /// the value is shaped like an address.
  @Validate(email: true, message: 'must be an email address')
  final String assignTo;

  /// Whether it starts out finished.
  ///
  /// The Dart constructor default covers a caller in Dart; `defaultValue`
  /// covers a request that omits the key. Both are needed, because
  /// deserialization builds the value without going through the default.
  @SerDe(defaultValue: false)
  final bool done;
}

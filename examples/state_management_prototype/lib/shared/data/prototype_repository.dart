import 'package:state_management_prototype/shared/api/prototype_api.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

abstract interface class PrototypeRepository {
  Future<RemoteUser> fetchOwner({required int userId});
  Future<List<RemoteTodo>> fetchTodos({required int userId, required int limit});
}

final class LivePrototypeRepository implements PrototypeRepository {
  LivePrototypeRepository(this._api);

  final PrototypeApi _api;

  @override
  Future<RemoteUser> fetchOwner({required int userId}) {
    return _api.fetchUser(userId);
  }

  @override
  Future<List<RemoteTodo>> fetchTodos({
    required int userId,
    required int limit,
  }) async {
    final todos = List<RemoteTodo>.from(
      await _api.listTodos(userId: userId, limit: limit),
    )
      ..sort((left, right) {
        if (left.completed != right.completed) {
          return left.completed ? 1 : -1;
        }
        return left.id.compareTo(right.id);
      });
    return todos;
  }
}

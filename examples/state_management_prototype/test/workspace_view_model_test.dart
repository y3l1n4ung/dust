import 'package:flutter_test/flutter_test.dart';
import 'package:state_management_prototype/features/session/view_models/session_view_model.dart';
import 'package:state_management_prototype/features/tasks/models/task_board_state.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

void main() {
  test('initialize loads repository data once', () async {
    final repository = FakePrototypeRepository();
    final viewModel = SessionViewModel(repository);

    await viewModel.initialize();
    await viewModel.initialize();

    expect(repository.calls, 1);
    expect(viewModel.value.owner?.name, 'Ada Lovelace');
    expect(viewModel.value.isLoading, isFalse);
  });

  test('query filter and toggle update derived state', () async {
    final viewModel = TaskBoardViewModel(FakePrototypeRepository());
    await viewModel.initialize();

    viewModel.setFilter(TodoFilter.open);
    viewModel.setQuery('design');

    expect(viewModel.value.visibleTodos.length, 1);
    expect(viewModel.value.visibleTodos.single.title, 'Review design QA');

    viewModel.toggleTodo(viewModel.value.visibleTodos.single.id);
    expect(viewModel.value.visibleTodos, isEmpty);
  });
}

final class FakePrototypeRepository implements PrototypeRepository {
  int calls = 0;

  @override
  Future<RemoteUser> fetchOwner({required int userId}) async {
    calls += 1;
    return const RemoteUser(
      id: 1,
      name: 'Ada Lovelace',
      username: 'ada',
      email: 'ada@dust.dev',
      phone: '555-0100',
      website: 'dust.dev',
      company: RemoteCompany(
        name: 'Dust Labs',
        catchPhrase: 'Ship the sharp edges first',
      ),
    );
  }

  @override
  Future<List<RemoteTodo>> fetchTodos({
    required int userId,
    required int limit,
  }) async {
    return const [
      RemoteTodo(
        userId: 1,
        id: 1,
        title: 'Review design QA',
        completed: false,
      ),
      RemoteTodo(
        userId: 1,
        id: 2,
        title: 'Ship release notes',
        completed: true,
      ),
      RemoteTodo(
        userId: 1,
        id: 3,
        title: 'Audit onboarding copy',
        completed: false,
      ),
    ];
  }
}

import 'package:flutter_test/flutter_test.dart';
import 'package:state_management_prototype/features/session/view_models/session_view_model.dart';
import 'package:state_management_prototype/features/tasks/models/task_board_state.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/models/remote_post.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

void main() {
  test('initialize loads repository data once', () async {
    final repository = FakePrototypeRepository();
    final viewModel = SessionViewModel(
      SessionViewModelArgs(repository: repository),
    );

    await viewModel.refresh();
    await viewModel.refresh();

    expect(repository.calls, 2);
    expect(viewModel.state.owner?.name, 'Ada Lovelace');
  });

  test('query filter and toggle update derived state', () async {
    final viewModel = TaskBoardViewModel(
      TaskBoardViewModelArgs(repository: FakePrototypeRepository()),
    );
    await viewModel.refresh();

    viewModel.setFilter(TodoFilter.open);
    viewModel.setQuery('design');

    expect(viewModel.state.visibleTodos.length, 1);
    expect(viewModel.state.visibleTodos.single.title, 'Review design QA');

    viewModel.toggleTodo(viewModel.state.visibleTodos.single.id);
    expect(viewModel.state.visibleTodos, isEmpty);
  });
}

final class FakePrototypeRepository implements PrototypeRepository {
  int calls = 0;

  @override
  Future<RemoteUser> fetchOwner({required int userId}) async {
    calls += 1;
    return RemoteUser(
      id: 1,
      name: 'Ada Lovelace',
      username: 'ada',
      email: 'ada@dust.dev',
      phone: '555-0100',
      website: 'dust.dev',
      company: const RemoteCompany(
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
    return [
      RemoteTodo(
        userId: 1,
        id: 1,
        title: 'Review design QA',
        completed: false,
        lane: 'Design',
        priority: 'High',
      ),
      RemoteTodo(
        userId: 1,
        id: 2,
        title: 'Ship release notes',
        completed: true,
        lane: 'Platform',
        priority: 'Medium',
      ),
      RemoteTodo(
        userId: 1,
        id: 3,
        title: 'Audit onboarding copy',
        completed: false,
        lane: 'Ops',
        priority: 'Low',
      ),
    ];
  }

  @override
  Future<List<RemotePost>> fetchPosts({int? userId}) async {
    return [
      RemotePost(userId: 1, id: 1, title: 'Post title 1', body: 'Post body 1'),
    ];
  }

  @override
  Future<RemotePost> fetchPost({required int postId}) async {
    return RemotePost(
      userId: 1,
      id: postId,
      title: 'Post title $postId',
      body: 'Post body $postId',
    );
  }
}

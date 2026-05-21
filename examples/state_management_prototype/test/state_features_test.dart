import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:state_management_prototype/features/tasks/models/task_board_state.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/models/remote_post.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

void main() {
  testWidgets('watch rebuilds only for accessed state aspect', (tester) async {
    final viewModel = TaskBoardViewModel(
      TaskBoardViewModelArgs(repository: _StaticRepository()),
    );
    var queryBuilds = 0;
    var filterBuilds = 0;

    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: TaskBoardViewModelScope.value(
          value: viewModel,
          child: Column(
            children: [
              _QueryProbe(onBuild: () => queryBuilds += 1),
              _FilterProbe(onBuild: () => filterBuilds += 1),
            ],
          ),
        ),
      ),
    );

    expect(queryBuilds, 1);
    expect(filterBuilds, 1);

    viewModel.setFilter(TodoFilter.open);
    await tester.pump();

    expect(queryBuilds, 1);
    expect(filterBuilds, 2);

    viewModel.setQuery('release');
    await tester.pump();

    expect(queryBuilds, 2);
    expect(filterBuilds, 2);
  });

  testWidgets('generated listener receives one-shot effects', (tester) async {
    final viewModel = TaskBoardViewModel(
      TaskBoardViewModelArgs(repository: _StaticRepository()),
    );
    final effects = <Object>[];

    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: TaskBoardViewModelScope.value(
          value: viewModel,
          child: TaskBoardViewModelListener(
            listener: (_, effect) => effects.add(effect),
            child: const SizedBox(),
          ),
        ),
      ),
    );

    viewModel.addTodo('Write listener test', 'QA', 'High');
    await tester.pump();

    expect(effects, hasLength(1));
    expect(effects.single, isA<TaskAddedEffect>());
    expect((effects.single as TaskAddedEffect).title, 'Write listener test');
  });

  testWidgets('generated scope reports args injection failures clearly', (
    tester,
  ) async {
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: TaskBoardViewModelScope(
          args: (_) => throw StateError('repository missing'),
          create: (_, args) => TaskBoardViewModel(args),
          child: const SizedBox(),
        ),
      ),
    );

    final error = tester.takeException();

    expect(error, isA<StateError>());
    expect(error.toString(), contains('TaskBoardViewModelScope'));
    expect(error.toString(), contains('repository missing'));
  });

  test('refresh ignores stale async completions', () async {
    final repository = _ControlledRepository();
    final viewModel = TaskBoardViewModel(
      TaskBoardViewModelArgs(repository: repository),
    );

    final firstRun = viewModel.refresh(showLoading: true);
    final secondRun = viewModel.refresh();

    repository.todoRequests[1].complete([
      const RemoteTodo(
        userId: 1,
        id: 2,
        title: 'newer result',
        completed: false,
      ),
    ]);
    await secondRun;

    repository.todoRequests[0].complete([
      const RemoteTodo(
        userId: 1,
        id: 1,
        title: 'stale result',
        completed: false,
      ),
    ]);
    await firstRun;

    expect(viewModel.state.todos.single.id, 2);
    expect(viewModel.state.todos.single.title, 'newer result');
  });
}

final class _QueryProbe extends StatelessWidget {
  const _QueryProbe({required this.onBuild});

  final VoidCallback onBuild;

  @override
  Widget build(BuildContext context) {
    onBuild();
    return Text(context.watchTaskBoardViewModel().query);
  }
}

final class _FilterProbe extends StatelessWidget {
  const _FilterProbe({required this.onBuild});

  final VoidCallback onBuild;

  @override
  Widget build(BuildContext context) {
    onBuild();
    return Text(context.watchTaskBoardViewModel().filter.name);
  }
}

final class _ControlledRepository implements PrototypeRepository {
  final todoRequests = <Completer<List<RemoteTodo>>>[];

  @override
  Future<RemoteUser> fetchOwner({required int userId}) =>
      throw UnimplementedError();

  @override
  Future<RemotePost> fetchPost({required int postId}) =>
      throw UnimplementedError();

  @override
  Future<List<RemotePost>> fetchPosts({int? userId}) =>
      throw UnimplementedError();

  @override
  Future<List<RemoteTodo>> fetchTodos({
    required int userId,
    required int limit,
  }) {
    final request = Completer<List<RemoteTodo>>();
    todoRequests.add(request);
    return request.future;
  }
}

final class _StaticRepository implements PrototypeRepository {
  @override
  Future<RemoteUser> fetchOwner({required int userId}) =>
      throw UnimplementedError();

  @override
  Future<RemotePost> fetchPost({required int postId}) =>
      throw UnimplementedError();

  @override
  Future<List<RemotePost>> fetchPosts({int? userId}) =>
      throw UnimplementedError();

  @override
  Future<List<RemoteTodo>> fetchTodos({
    required int userId,
    required int limit,
  }) async {
    return const [];
  }
}

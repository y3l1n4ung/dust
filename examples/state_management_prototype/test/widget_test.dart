import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:state_management_prototype/app/prototype_app.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/data/state_observer.dart';
import 'package:state_management_prototype/shared/models/remote_post.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

void main() {
  testWidgets('prototype renders dashboard and task search', (tester) async {
    final repository = FakePrototypeRepository();

    await tester.pumpWidget(
      PrototypeApp(
        repository: repository,
        observer: const SilentStateObserver(),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.text('Project Dashboard'), findsOneWidget);
    expect(find.text('Project coordination for Dust Labs'), findsOneWidget);

    await tester.tap(find.text('Tasks'));
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), 'release');
    await tester.pumpAndSettle();

    expect(find.text('Ship release notes'), findsOneWidget);
    expect(find.text('Audit onboarding copy'), findsNothing);
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
      RemoteTodo(userId: 1, id: 1, title: 'Review design QA', completed: false),
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

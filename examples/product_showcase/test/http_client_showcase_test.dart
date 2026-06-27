import 'package:dio/dio.dart';
import 'package:product_showcase/product_showcase.dart';
import 'package:test/test.dart';

void main() {
  test('todo serde roundtrip stays intact', () {
    final todo = Todo(
      id: 'todo-1',
      title: 'Ship HttpClient support',
      isCompleted: false,
    );
    final decoded = Todo.fromJson(todo.toJson());

    expect(decoded.id, todo.id);
    expect(decoded.title, todo.title);
    expect(decoded.isCompleted, todo.isCompleted);
    expect(TodoCreate(title: 'Write docs', isCompleted: true).toJson(), {
      'title': 'Write docs',
      'isCompleted': true,
    });
    expect(
      TodoUpdate(
        title: 'Refine render reuse',
      ).copyWith(isCompleted: true).toJson(),
      {'title': 'Refine render reuse', 'isCompleted': true},
    );
  });

  test('todo API keeps query and header defaults', () async {
    RequestOptions? captured;
    final dio = Dio();
    dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) {
          captured = options;
          handler.resolve(
            Response<dynamic>(requestOptions: options, data: const <dynamic>[]),
          );
        },
      ),
    );

    await TodoApi(dio).list(userId: 7);

    final request = captured!;
    expect(request.queryParameters['userId'], 7);
    expect(request.queryParameters['page'], 1);
    expect(request.headers['x-trace-id'], 'showcase-default');
  });
}

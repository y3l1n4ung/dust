import 'package:dust_benchmark_project/support/http_post.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('http support models round-trip through serde helpers', () {
    final post = HttpPost(
      userId: 7,
      id: 21,
      title: 'Benchmark fixture',
      body: 'HTTP generation now participates in the large corpus.',
    );

    expect(HttpPost.fromJson(post.toJson()).toJson(), equals(post.toJson()));
  });
}

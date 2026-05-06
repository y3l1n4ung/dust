import 'package:dust_stress_project/support/http_api.test.g.dart'
    as http_generated;
import 'package:dust_stress_project/support/http_post.dart';
import 'package:test/test.dart';

void main() {
  http_generated.main();

  test('http support models round-trip through serde helpers', () {
    final post = HttpPost(
      userId: 7,
      id: 21,
      title: 'Stress fixture',
      body: 'HTTP generation now participates in the large corpus.',
    );

    expect(HttpPost.fromJson(post.toJson()).toJson(), equals(post.toJson()));
  });
}

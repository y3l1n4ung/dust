import 'package:dust_dart/http.dart';
import 'package:test/test.dart';

void main() {
  test('HTTP client annotations preserve constructor options', () {
    const client = HttpClient(
      baseUrl: 'https://api.example.com',
      target: DustHttpTarget.flutter,
      parseThread: DustParseThread.isolate,
      headers: {'x-api': 'dust'},
    );
    const generatedTests = GenerateTest();
    const parse = HttpParse(thread: DustParseThread.isolate);

    expect(client.baseUrl, 'https://api.example.com');
    expect(client.target, DustHttpTarget.flutter);
    expect(client.parseThread, DustParseThread.isolate);
    expect(client.headers, {'x-api': 'dust'});
    expect(generatedTests, isA<GenerateTest>());
    expect(parse.thread, DustParseThread.isolate);
  });

  test('HTTP method annotations expose paths', () {
    const methods = <HttpMethod>[
      GET('/users'),
      POST('/users'),
      PUT('/users/{id}'),
      PATCH('/users/{id}'),
      DELETE('/users/{id}'),
      HEAD('/users/{id}'),
      OPTIONS('/users'),
    ];

    expect(methods.map((method) => method.path), [
      '/users',
      '/users',
      '/users/{id}',
      '/users/{id}',
      '/users/{id}',
      '/users/{id}',
      '/users',
    ]);
  });

  test('HTTP parameter annotations preserve names and keys', () {
    const path = Path('id');
    const unnamedPath = Path();
    const query = Query('q');
    const queries = Queries();
    const header = Header('authorization');
    const headers = Headers({'x-api': 'dust'});
    const headerMap = HeaderMap();
    const body = Body();
    const field = Field('name');
    const part = Part('avatar');
    const extra = Extra('traceId');
    const form = FormUrlEncoded();
    const multipart = MultiPart();

    expect(path.name, 'id');
    expect(unnamedPath.name, isNull);
    expect(query.name, 'q');
    expect(queries, isA<Queries>());
    expect(header.name, 'authorization');
    expect(headers.values, {'x-api': 'dust'});
    expect(headerMap, isA<HeaderMap>());
    expect(body, isA<Body>());
    expect(field.name, 'name');
    expect(part.name, 'avatar');
    expect(extra.key, 'traceId');
    expect(form, isA<FormUrlEncoded>());
    expect(multipart, isA<MultiPart>());
  });
}

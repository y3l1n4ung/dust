import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';
import 'package:test/test.dart';

void main() {
  group('HttpClient Annotation Tests', () {
    test('HttpClient properties', () {
      const client = HttpClient(
        baseUrl: 'https://api.example.com',
        target: DustHttpTarget.flutter,
        parseThread: DustParseThread.isolate,
        headers: {'x-client': 'dust'},
      );
      expect(client.baseUrl, 'https://api.example.com');
      expect(client.target, DustHttpTarget.flutter);
      expect(client.parseThread, DustParseThread.isolate);
      expect(client.headers, {'x-client': 'dust'});
    });

    test('GenerateTest instantiation', () {
      const generateTest = GenerateTest();
      expect(generateTest, isA<GenerateTest>());
    });
  });

  group('HTTP Verb Annotations', () {
    test('GET', () {
      const get = GET('/users');
      expect(get.path, '/users');
    });

    test('POST', () {
      const post = POST('/users');
      expect(post.path, '/users');
    });

    test('PUT', () {
      const put = PUT('/users/{id}');
      expect(put.path, '/users/{id}');
    });

    test('PATCH', () {
      const patch = PATCH('/users/{id}');
      expect(patch.path, '/users/{id}');
    });

    test('DELETE', () {
      const delete = DELETE('/users/{id}');
      expect(delete.path, '/users/{id}');
    });

    test('HEAD', () {
      const head = HEAD('/ping');
      expect(head.path, '/ping');
    });

    test('OPTIONS', () {
      const options = OPTIONS('*');
      expect(options.path, '*');
    });
  });

  group('Parameter Annotations', () {
    test('Path', () {
      const pathNamed = Path('userId');
      const pathUnnamed = Path();
      expect(pathNamed.name, 'userId');
      expect(pathUnnamed.name, isNull);
    });

    test('Query', () {
      const query = Query('q');
      expect(query.name, 'q');
    });

    test('Queries', () {
      const queries = Queries();
      expect(queries, isA<Queries>());
    });

    test('Header', () {
      const header = Header('x-request-id');
      expect(header.name, 'x-request-id');
    });

    test('Headers', () {
      const headers = Headers({'Content-Type': 'application/json'});
      expect(headers.values, {'Content-Type': 'application/json'});
    });

    test('HeaderMap', () {
      const headerMap = HeaderMap();
      expect(headerMap, isA<HeaderMap>());
    });

    test('Body', () {
      const body = Body();
      expect(body, isA<Body>());
    });

    test('Field', () {
      const field = Field('username');
      expect(field.name, 'username');
    });

    test('Part', () {
      const part = Part('file');
      expect(part.name, 'file');
    });

    test('Extra', () {
      const extra = Extra('cache');
      expect(extra.key, 'cache');
    });
  });

  group('Specialized Annotations', () {
    test('FormUrlEncoded', () {
      const form = FormUrlEncoded();
      expect(form, isA<FormUrlEncoded>());
    });

    test('MultiPart', () {
      const multipart = MultiPart();
      expect(multipart, isA<MultiPart>());
    });

    test('HttpParse override', () {
      const parse = HttpParse(thread: DustParseThread.isolate);
      expect(parse.thread, DustParseThread.isolate);
    });
  });
}

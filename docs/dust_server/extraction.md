# Extraction

An extractor turns a request into one value a handler wants, or a `Rejection`
explaining why it could not. The interfaces follow axum: `FromRequestParts` for
anything that does not read the body, `FromRequest` for anything that does.

## Reading from a handler

A handler reads values straight off the request. Each call throws the rejection
its extractor produced, so the first failure ends the endpoint and the verb
builder turns it into the response:

```dart
Future<Result<Todo, Rejection>> readTodo(Request request) async {
  final user = await request.extract<AuthUser>(const BearerAuth());
  final id = await request.path<String>('id');
  final repository = await request.state<TodoRepository>();

  final todo = repository.find(id);
  return todo == null ? const Err(_missing) : Ok(todo);
}
```

| From the request | Reads |
| :--- | :--- |
| `path<T>(name)` | a captured path segment, percent-decoded and coerced |
| `query<T>(name)` | one query value; a nullable `T` makes it optional |
| `queryList<T>(name)` | every value of a repeated key |
| `queries()` | the whole query map |
| `rawQuery()` | the undecoded query string |
| `header(name)` | one header, matched case-insensitively |
| `headerMap()` | every header, lower-cased |
| `cookie<T>(name)` | one cookie, coerced |
| `cookies()` | every cookie, as a `CookieJar` |
| `bearerToken()` | the token from `Authorization: Bearer` |
| `basicCredentials()` | the decoded user and password from `Authorization: Basic` |
| `apiKey()` | `X-API-Key`, falling back to `?api_key=` |
| `sessionId()` | the `session` cookie, as 401 rather than 400 |
| `body<T>(deserialize)` | a JSON object body |
| `bodyList<T>(deserialize)` | a JSON array body |
| `validBody<T>(deserialize)` | a JSON object body, then its `Validate()` constraints |
| `textBody()` | UTF-8 text |
| `rawBody()` | bytes |
| `bodyStream()` | the unread body stream |
| `form()` | a urlencoded body, or the query on `GET` and `HEAD` |
| `multipart()` | a multipart body, buffered |
| `state<T>()` | application state attached with `withState` |
| `peer()` | the connection's address and port |
| `extract(anything)` | any extractor, including your own |

`extract` is the only one whose result is not written at the call site, so pass
the type: `request.extract<AuthUser>(...)`. An authenticating extractor returns
a caller, and checking that someone is signed in without checking *who* is half
a check.

### Trying it

```bash
dart run example/path_params.dart
```

```bash
curl -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos
curl -i -H 'authorization: Bearer todos:read' 'localhost:8080/api/v1/todos?done=maybe'
curl -i localhost:8080/api/v1/todos
```

The second is a 400 — `done` will not coerce to `bool`. The third is a 401 with
`WWW-Authenticate: Bearer`.

## The classes underneath

Every method above is a class, which is what a generator names and composes.
Use the class when you want the `Result` rather than the throw.

| Extractor | Shortcut |
| :--- | :--- |
| `PathExtractable<T>` | `path<T>(name)` |
| `QueryExtractable<T>`, `QueryListExtractable<T>`, `QueriesExtractable<T>`, `RawQueryExtractable` | `query`, `queryList`, `queries`, `rawQuery` |
| `HeaderExtractable`, `HeaderMapExtractable` | `header`, `headers` |
| `CookieExtractable<T>`, `CookieJarExtractable` | `cookie<T>`, `cookies` |
| `BearerTokenExtractable`, `BasicCredentialsExtractable`, `ApiKeyExtractable`, `SessionIdExtractable` | `bearerToken`, `basicCredentials`, `apiKey`, `sessionId` |
| `JsonExtractable<T>`, `JsonListExtractable<T>` | `body<T>`, `bodyList<T>` |
| `FormExtractable`, `MultipartExtractable` | `form`, `multipart` |
| `RawBodyExtractable`, `TextBodyExtractable`, `StreamBodyExtractable` | `rawBody`, `textBody`, `bodyStream` |
| `StateExtractable<T>`, `ContextExtractable<T>`, `PeerExtractable` | `state<T>`, —, `peer` |

## Coercion

Keyed extractors coerce to `String`, `int`, `double`, `num`, `bool`, `BigInt`,
`DateTime`, `Uri`, and their nullable forms. `bool` accepts `true`, `false`,
`1`, `0`, and a bare flag. Anything else is a programming error and throws
rather than rejecting.

Integers are **decimal only**. `int.tryParse` reads `0x10` as 16 by default,
which would make `?id=0x10` and `?id=16` name the same record while a check
written against the text sees two different strings.

## Wrapping

```dart
const viewer = OptionalExtractable(BearerAuth());   // Option<AuthUser>
const outcome = FallibleExtractable(BearerAuth());  // Result<AuthUser, Rejection>
const input = ValidatedExtractable(JsonExtractable(CreateTodo.deserialize));
const signedIn = FirstOf<Caller>([SessionScheme(), BearerScheme()]);
```

`Option` collapses **client errors only**. A 5xx propagates, because those
report a server-side fault, and hiding one would turn a misconfiguration into a
silently anonymous request. `FirstOf` stops at a 5xx for the same reason.

`readsRequestBody` looks through all of these, so a composition check sees the
body extractor underneath a wrapper.

## Writing one

```dart
final class BearerAuth implements FromRequestParts<AuthUser> {
  const BearerAuth({this.scope});

  final String? scope;

  @override
  Future<Result<AuthUser, Rejection>> extract(Request request) async {
    // The class, not `request.bearerToken()`: inside an `extract` the failure
    // has to come back as `Err` so a combinator can try the next one.
    switch (await const BearerTokenExtractable().extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final token):
        ...
    }
  }
}

final class TodosWrite extends BearerAuth {
  const TodosWrite() : super(scope: 'todos:write');
}
```

**Return the failure; do not throw it.** The `request.x()` methods throw, which
is right in a handler and wrong here: a combinator reads the `Result` to decide
whether to try the next extractor, and a throw sails past it and ends the
request.

Configuration is expressed by subclassing, the way axum encodes it in the type,
because an extractor named in `@Extract` needs a zero-argument `const`
constructor.

## Bodies

A body can only be read once, so at most one body extractor runs per handler and
it runs last. `FormExtractable` and `MultipartExtractable` read the body once and
hand back a `FormMap` or `MultipartForm`, which every `@Field` and `@Part` then
reads from.

`readBody` refuses an oversized `content-length` before reading anything, and
cuts off a streamed body at the limit. A limit set on the root router reaches
extractors whose own limit was fixed at build time.

```bash
# 415: the extractor wanted JSON
curl -i -X POST localhost:8080/api/v1/todos \
  -H 'authorization: Bearer todos:write' \
  -H 'content-type: text/plain' --data 'not json'

# 400: JSON that does not parse
curl -i -X POST localhost:8080/api/v1/todos \
  -H 'authorization: Bearer todos:write' \
  -H 'content-type: application/json' --data '{'

# 422: parses, but breaks a constraint
curl -i -X POST localhost:8080/api/v1/todos \
  -H 'authorization: Bearer todos:write' \
  -H 'content-type: application/json' --data '{"title":""}'
```

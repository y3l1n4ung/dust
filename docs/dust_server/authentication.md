# Authentication

The runtime **extracts credentials and stops there**. What a caller is, and what
a scope permits, is a product decision — so nothing here asks your user type to
implement an interface it did not choose.

Working examples: `example/bearer_auth.dart` for one scheme, and
`example/credential_schemes.dart` for accepting whichever of four arrives.

## What the runtime reads

| Extractor | Reads | Refuses with |
| :--- | :--- | :--- |
| `BearerTokenExtractable` | `Authorization: Bearer <token>` | 401, `WWW-Authenticate: Bearer` |
| `BasicCredentialsExtractable` | `Authorization: Basic base64(user:pass)` | 401, `WWW-Authenticate: Basic realm="…"` |
| `ApiKeyExtractable` | `X-API-Key`, or `?api_key=` | 401, `WWW-Authenticate: ApiKey` |
| `SessionIdExtractable` | the `session` cookie | 401, `WWW-Authenticate: Cookie` |
| `Authorization.of(request)` | the raw scheme/credentials split | — |

The scheme is matched **case-insensitively**. A client sending `bearer` rather
than `Bearer` is conforming, and an extractor that misses it fails for one SDK
and nobody else.

Answering the right challenge matters: a Basic endpoint replying
`WWW-Authenticate: Bearer` is why a browser sometimes shows no password prompt
at all.

## Turning a credential into a caller

That part is yours. A scheme is the runtime's extractor plus one lookup:

```dart
final class BearerScheme implements FromRequestParts<Caller> {
  const BearerScheme();

  @override
  Future<Result<Caller, Rejection>> extract(Request request) async {
    switch (await const BearerTokenExtractable().extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final token):
        final credentials = await request.state<Credentials>();
        final subject = credentials.bearerTokens[token];
        return subject == null
            ? const Err(Rejection.unauthorized('unknown token'))
            : Ok(Caller(subject, scopes: const {'read', 'write'}));
    }
  }
}
```

Use the **class**, not `request.bearerToken()`. Inside an `extract` the failure
has to come back as `Err`, or a combinator cannot try the next scheme.

## Accepting more than one scheme

`FirstOf` tries extractors in order and takes the first that succeeds:

```dart
const signedIn = FirstOf<Caller>([
  BearerScheme(),
  ApiKeyScheme(),
  BasicScheme(),
  SessionScheme(),
]);
```

Two rules worth knowing:

- The rejection reported is the **last** one tried, so put the scheme your
  callers most likely meant last. Telling a browser user their bearer token is
  missing helps nobody.
- A **5xx stops the search**. A server fault is not a reason to ask for other
  credentials, and turning it into a 401 would hide a broken deployment.

## Authorization

A wrapper, in your code, so a new scheme gets it for free:

```dart
final class RequireScope implements FromRequestParts<Caller> {
  const RequireScope(this.inner, this.scope);

  final FromRequestParts<Caller> inner;
  final String scope;

  @override
  Future<Result<Caller, Rejection>> extract(Request request) async {
    switch (await inner.extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(:final value):
        return value.can(scope)
            ? Ok(value)
            : Err(Rejection.forbidden('requires the $scope scope'));
    }
  }
}
```

**403, not 401.** They proved who they are; authenticating again would not help.

## Trying it

```bash
dart run example/bearer_auth.dart
```

```bash
# bearer
curl -s -H 'authorization: Bearer t-ada' localhost:8082/whoami

# api key, in the header and in the query
curl -s -H 'x-api-key: k-robot' localhost:8082/whoami
curl -s 'localhost:8082/whoami?api_key=k-robot'

# basic
curl -s -u ada:secret localhost:8082/whoami

# session cookie, alongside others
curl -s -H 'cookie: theme=dark; session=s-ada' localhost:8082/whoami
```

Each answers with the subject and which scheme carried it:

```json
{"id":"ada@dust.test","via":"bearer","scopes":["read","write"]}
```

The refusals are the interesting part:

```bash
# 401, and the challenge a browser needs
curl -i localhost:8082/basic

# 401 naming the scheme tried last, not first
curl -i localhost:8082/whoami

# 403: the API key authenticated, but grants only read
curl -i -H 'x-api-key: k-robot' localhost:8082/admin

# 200: the bearer token grants write
curl -i -H 'authorization: Bearer t-ada' localhost:8082/admin
```

## Notes worth heeding

**An API key in a URL lands in access logs, proxy logs, and browser history.**
`ApiKeyExtractable` accepts one because existing clients send it, but
`allowQuery: false` is a single flag, and the example grants query-authenticated
callers read-only scopes to make the cost visible.

**Compare secrets carefully.** The example compares passwords with `==`, which
is fine for a demo and wrong for production: use a constant-time comparison, and
store a hash rather than the password.

**A 401 challenge reaches a response header.** It is the only part of a
`Rejection` that does — control characters are stripped so a realm built from
anything a client influenced cannot split the response.

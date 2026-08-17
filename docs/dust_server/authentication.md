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

One scheme first — `dart run example/bearer_auth.dart`:

```bash
curl -s  localhost:8080/me -H 'authorization: Bearer t-ada'
curl -si localhost:8080/me
curl -si localhost:8080/me -H 'authorization: Basic abc'
curl -si localhost:8080/me -H 'authorization: Bearer nope'
```

```
200 {"user":"ada"}
401 WWW-Authenticate: Bearer   {"error":"expected a bearer token"}
401 WWW-Authenticate: Bearer   {"error":"expected a bearer token"}
403                            {"error":"unknown token"}
```

The last two are the pair to read. A wrong **scheme** is a 401 — no credential
was presented, so presenting one may work. A wrong **token** is a 403 — one was
presented and it is not allowed, so retrying is pointless. Answer 401 for both
and a client that retries on 401 loops forever.

Four schemes behind one interface — `dart run example/credential_schemes.dart`:

```bash
curl -s localhost:8080/whoami -H 'authorization: Bearer t-ada'
curl -s localhost:8080/whoami -H 'x-api-key: k-robot'
curl -s localhost:8080/whoami -u ada:secret
curl -s localhost:8080/whoami -H 'cookie: session=s-ada'
```

```json
{"via":"bearer","id":"ada"}
{"via":"api-key","id":"robot"}
{"via":"basic","id":"ada"}
{"via":"session","id":"ada"}
```

The handler never learns which header carried the credential, which is the point
of `firstOf`. Two refusals are worth seeing:

```bash
curl -si 'localhost:8080/whoami?api_key=k-robot'
curl -si  localhost:8080/whoami
```

```
401 WWW-Authenticate: Cookie   {"error":"expected a session cookie"}
401 WWW-Authenticate: Cookie   {"error":"expected a session cookie"}
```

The first is `allowQuery: false` doing its job — a key in a URL is refused even
though it is the right key.

The second is a wart, and the reason it is documented rather than hidden:
`firstOf` returns the **last** `Err`, so the challenge and the message come from
whichever scheme ran last. `expected a session cookie` is accurate for that one
extractor and misleading about the other three. HTTP allows several challenges in
one response; if the message matters, order the list so the most likely scheme
runs last, or answer the 401 yourself instead of letting the last extractor do it.

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

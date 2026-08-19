# Dependencies

Every dependency here replaces code this package would otherwise have to write
and maintain. The rule is that a mature, widely used package beats a hand-rolled
version, and the exceptions are recorded rather than assumed.

## What is used

| Package | Purpose | Why it, specifically |
| :--- | :--- | :--- |
| `shelf` | `Handler`, `Request`, `Response`, `Middleware` | the interoperability contract the Dart server ecosystem speaks |
| `shelf_static` | static files, single-page fallback | content types, ranges, conditional requests, and directory escape already handled; 8.4M downloads a month |
| `shelf_web_socket` | the WebSocket handshake | the standard upgrade for `shelf` |
| `web_socket_channel` | the socket itself | what `shelf_web_socket` yields, and what clients use |
| `mustache_template` | templating | the most used template package on pub, 7.9M downloads a month |
| `http_parser` | `content-type` and media type parsing | the same parser the rest of the Dart HTTP stack uses; parameters, casing, and quoting are not worth re-deriving |
| `mime` | multipart decoding | the Dart team's implementation of a format with real edge cases |
| `uuid` | request ids | ids unique across processes and machines, not just within one isolate |
| `dust_dart` | `Result`, `Option` | the workspace's own functional types |
| `meta` | `@internal`, `@immutable` | keeps internals out of the public surface |

Dev-only, and each earns its place:

| Package | Used by |
| :--- | :--- |
| `test` | the suite |
| `http` | a real client, for the tests that go over a socket |
| `coverage` | `scripts/dart/coverage.sh`, which fails the build under 100% |
| `shelf_router` | the conformance oracle in `test/router/conformance/` |
| `jinja` | a second `TemplateEngine`, proving the interface is pluggable rather than mustache with extra steps |
| `crypto` | `example/sessions.dart` and `example/webhook_signatures.dart` — signing needs HMAC, and hand-rolling one is never the answer |
| `very_good_analysis` | the lint set, via the root `analysis_options.yaml` |

None of them is a runtime dependency: `lib/` imports none of the above.

## What was deliberately not used

**`shelf_router`, as a runtime dependency.** It was doing only the final lookup,
while this package already computed 405s, path patterns, and HEAD fallback.
Worse, retrieving path parameters meant reading its private
`shelf_router/params` context key.

It remains a **dev dependency** on purpose. `test/router/conformance/` checks
this router against it on 24 hand-picked and 1000 generated paths, and
`test/router/integration/` mounts one inside the other in both directions. Using
the package it replaces as the oracle is the cheapest evidence available that
the replacement is correct.

**An OpenAPI package.** `open_api` is abandoned (2018, Dart 2). `openapi_spec`
is pre-1.0 with near-zero adoption. The rest generate clients from a spec, the
opposite direction. Documentation is deferred to a separate package instead, and
`Router.describe()` plus an opaque `metadata` slot is the seam it will build on.

**A hand-rolled template engine, HTML escaper, media-type parser, and id
generator.** All four existed here briefly and all four were deleted in favour of
the packages above. A template engine in particular is a parser, and a parser
written in an afternoon is a liability.

## What stays hand-rolled, and why

**Route matching.** See above: the alternative coupled us to another package's
private key.

**Type coercion** (`String` to `int`, `bool`, `DateTime`, and so on). No package
does exactly this, and the alternative is each extractor parsing by hand.

**Path normalization.** The rules are specific to how prefixes join and how a
trailing slash is treated, and they are twenty lines with their own tests.

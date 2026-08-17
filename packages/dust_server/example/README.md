# Examples

Two sizes, and the small one is the point.

**One-question examples** are single files under 60 lines. Each answers exactly
one question and nothing else, so the answer is not buried in an application.
Named after the question you would type into a search box — `handle_head_request`,
not `advanced_routing`.

**Complete applications** come later, and there will be few of them. They show
the pieces working together, which no single-concept file can — and that is the
only job they have.

## The contract

Every example in this directory:

1. Opens with a doc comment saying what it answers, and how to run it:
   `dart run example/cors.dart`
2. Carries a `curl` block for the endpoints it serves, so the example can be
   checked from a terminal rather than only by reading it.
3. Splits `Router buildApp()` out of `main`, so the shared suite in
   [`../test/example/`](../test/example) can serve it on a loopback port
   without a process.
4. Answers one question. A second concept in the file belongs in a second file.

`buildApp()` takes no arguments — anything the example needs, it constructs. A
complete application takes its dependencies instead, so a test can substitute a
repository.

## Routing and requests

| Question | File |
| :--- | :--- |
| The smallest server that answers | [`hello_world.dart`](hello_world.dart) |
| Paths, methods, nesting, merging | [`routing.dart`](routing.dart) |
| Reading `{id}` out of the path | [`path_params.dart`](path_params.dart) |
| Reading the query string, including repeated keys | [`query_params.dart`](query_params.dart) |
| Reading headers, and the `Host` the client claims | [`headers_and_host.dart`](headers_and_host.dart) |
| Reading and setting cookies | [`cookies.dart`](cookies.dart) |
| Answering `HEAD` from a `GET` handler | [`handle_head_request.dart`](handle_head_request.dart) |
| One fallback for everything unmatched | [`global_404.dart`](global_404.dart) |
| Serving two versions of the same API | [`versioning.dart`](versioning.dart) |

## Bodies and validation

| Question | File |
| :--- | :--- |
| Decoding a JSON body into a model | [`json_body.dart`](json_body.dart) |
| Decoding an HTML form post | [`form_body.dart`](form_body.dart) |
| Handling a file upload | [`multipart_form.dart`](multipart_form.dart) |
| Answering 422 with the per-field errors | [`validation_422.dart`](validation_422.dart) |
| Changing what a rejection looks like on the wire | [`customize_rejection.dart`](customize_rejection.dart) |
| Choosing a decoder by `content-type` | [`parse_body_by_content_type.dart`](parse_body_by_content_type.dart) |

## Extraction

| Question | File |
| :--- | :--- |
| Passing a database or client to a handler | [`state.dart`](state.dart) |
| Writing an extractor of your own | [`custom_extractor.dart`](custom_extractor.dart) |
| Making a missing value `None` instead of a 400 | [`optional_extraction.dart`](optional_extraction.dart) |
| Handing the failure to the handler instead of short-circuiting | [`fallible_extraction.dart`](fallible_extraction.dart) |
| Requiring a bearer token | [`bearer_auth.dart`](bearer_auth.dart) |
| Accepting an API key, a cookie, or Basic — whichever arrives | [`credential_schemes.dart`](credential_schemes.dart) |

## Layers

| Question | File |
| :--- | :--- |
| Allowing a browser on another origin | [`cors.dart`](cors.dart) |
| Compressing responses | [`compression.dart`](compression.dart) |
| Giving every request an id that reaches the logs | [`request_id.dart`](request_id.dart) |
| Recording what was served | [`access_log.dart`](access_log.dart) |
| Making `/todos` and `/todos/` the same route | [`normalize_path.dart`](normalize_path.dart) |
| The response headers a browser uses to lock a page down | [`security_headers.dart`](security_headers.dart) |
| A layer that runs only for routes that matched | [`route_layer.dart`](route_layer.dart) |
| Cutting off a request that takes too long | [`request_timeout.dart`](request_timeout.dart) |

## Responses

| Question | File |
| :--- | :--- |
| Redirecting, and which status to use | [`redirects.dart`](redirects.dart) |
| Streaming events to a browser | [`sse.dart`](sse.dart) |
| A WebSocket on the same router as the HTTP routes | [`websockets.dart`](websockets.dart) |
| Rendering HTML from a template | [`templates.dart`](templates.dart) |
| Serving a built front end, single-page routes included | [`static_files.dart`](static_files.dart) |

## Operations

| Question | File |
| :--- | :--- |
| Shutting down without dropping requests in flight | `graceful_shutdown.dart` |
| Spans, and continuing a trace that started upstream | `tracing.dart` |
| Using every core | `clustered_isolates.dart` |
| Serving over TLS | `tls.dart` |
| Counting requests and timing them | `metrics.dart` |
| Signed sessions over the cookie extractor | `sessions.dart` |
| Testing an application built on this runtime | `testing.dart` |

`metrics.dart` and `sessions.dart` are examples rather than runtime features on
purpose. Both are policy: what to count, and what a session means. The runtime
supplies the layer and the cookie extractor and stops there — the same boundary
that keeps authorization out of it.

## Running them

```bash
dart run example/hello_world.dart
```

Every one of them is served over a real socket by
[`../test/example/`](../test/example). An example that stops compiling fails the
suite, which is what keeps a directory this size from rotting.

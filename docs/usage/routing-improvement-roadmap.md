# Routing Improvement Roadmap

This roadmap records suggested Dust router improvements after comparing the
current API with Flutter Router, go_router, AutoRoute, and Beamer. The design
goal stays the same: keep routing easy to remember with `@AppRouter` and
`@AppRoute`; prefer router overrides, generated helpers, and diagnostics before
adding new annotations.

## Priority Summary

| Priority | Area | Goal | Human DX impact | Tracking |
| :--- | :--- | :--- | :--- | :--- |
| P0 | Full deep-link support | Make every incoming app, universal, and browser link parse or fail predictably. | Users can trust copied URLs, push links, and cold-start links. | [#407](https://github.com/y3l1n4ung/dust/issues/407) |
| P0 | Full web URL support | Make hash URLs, path URLs, browser refresh, back, forward, query, and fragment behavior explicit and tested. | Web apps feel normal and debuggable. | [#407](https://github.com/y3l1n4ung/dust/issues/407) |
| P1 | Branch and shell parity | Keep tab stacks understandable without copying go_router's `StatefulShellRoute` complexity. | Bottom tabs preserve state without new annotations. | [#328](https://github.com/y3l1n4ung/dust/issues/328) |
| P1 | URL parameter codecs | Support more real URL value types without handwritten parsing. | Route constructors stay typed for product-grade links. | New issue when scoped |
| P1 | Route diagnostics | Explain redirects, guards, shell selection, branch selection, and URL parsing in one trace. | Developers fix routing bugs from logs instead of generated internals. | Existing routing diagnostics work |
| P2 | Route graph tooling | Generate a route graph and route table report from `dust doctor` or a future route subcommand. | Humans and AI agents can inspect the app's navigation map quickly. | New issue when scoped |

## Full Deep-Link Support

| Use case | Current Dust behavior | Suggested improvement | Acceptance test |
| :--- | :--- | :--- | :--- |
| Cold-start link opens `/products/42?tab=reviews` | Generated parser builds the typed route when Flutter gives the URI to the router. | Add documented platform checklist for Android app links, iOS universal links, and web links. | App starts from a platform URI and commits `ProductRoute(id: 42, tab: 'reviews')`. |
| Warm-link while app is running | Router can parse route information updates through Flutter Router. | Add runtime tests for receiving a second platform URL after the initial stack exists. | Existing stack changes to the incoming typed route without losing result futures incorrectly. |
| Auth redirect preserves target | Router examples preserve `route.location` manually. | Provide a first-class redirect-back recipe and regression tests for query and fragment preservation. | `/checkout?coupon=A#pay` redirects to login and returns to the exact target. |
| Unknown host or unsafe external link | Docs mention app code should reject untrusted hosts. | Add a router-level `normalizeIncomingUri(Uri uri)` override before parsing. | Unknown host maps to not-found or a safe public route. |
| Marketing or email links with prefixes | Not first-class today. | Add a URI normalization recipe for stripping prefixes such as `/app` or `/invite`. | `/app/products/42` normalizes to `/products/42` before matching. |
| Invalid path parameter | Invalid typed values go to not-found. | Keep behavior, but log parameter name and expected type in diagnostics. | `/products/not-int` logs expected `int productId` and commits not-found. |

Recommended API shape:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class RootRouter extends $RootRouter {
  @override
  Uri normalizeIncomingUri(Uri uri) {
    if (uri.host.isNotEmpty && uri.host != 'example.com') {
      return Uri(path: '/404', queryParameters: {'path': uri.toString()});
    }
    return uri.path.startsWith('/app')
        ? uri.replace(path: uri.path.substring('/app'.length))
        : uri;
  }
}
```

Why this shape: it improves deep-link handling without a new annotation. The
router remains the place for app-wide URL policy.

## Full Web URL Support

| Web behavior | Current Dust behavior | Suggested improvement | Acceptance test |
| :--- | :--- | :--- | :--- |
| Hash URL strategy | Works through Flutter's default URL strategy. | Document exact expected examples: `/#/products/42` and `/#/tabs/home`. | Browser address bar hash path round-trips through `parseAppRoute`. |
| Path URL strategy | Docs show `usePathUrlStrategy()`. | Add hosting rewrite examples for common static hosts. | Refreshing `/products/42` loads `index.html` and restores route. |
| Query strings | Supported for primitive route parameters. | Add explicit tests for unknown query preservation and default-valued query omission. | `/products/42?tab=reviews&x=1` keeps `x=1` in `location`. |
| Fragments | Fragment is preserved in route location. | Define whether generated route constructors should expose fragment or only preserve it. | `/docs/setup#step-2` round-trips exactly. |
| Browser back and forward | Runtime has route stack behavior and web-history tests are tracked. | Expand tests for branch swaps, guarded redirects, same-location replaces, and query-only changes. | Back/forward produces the same typed stack as direct parsing. |
| Base href and subdirectory deploys | Not first-class in Dust docs. | Document `/app/` deploys and pair with `normalizeIncomingUri`. | App served under `/app/` refreshes `/app/products/42` correctly. |
| 404 fallback hosting | Docs mention rewriting unknown app paths to `index.html`. | Add provider-specific examples for Nginx, Firebase Hosting, GitHub Pages, Cloudflare Pages, and Vercel. | Copy-paste config works in a sample web app. |

Recommended docs table to add to app guides:

| Deployment target | Required rewrite |
| :--- | :--- |
| Nginx | `try_files $uri $uri/ /index.html;` |
| Firebase Hosting | Rewrite `**` to `/index.html`. |
| Vercel | Rewrite `/(.*)` to `/index.html` for Flutter app paths. |
| Cloudflare Pages | Add an SPA fallback or `_redirects` rule to `/index.html`. |
| GitHub Pages | Use hash URLs or add a custom 404 fallback strategy. |

## Branch And Shell Improvements

| Problem | Suggested improvement | Why it improves DX | Avoid |
| :--- | :--- | :--- | :--- |
| Branch names are strings. | Generate branch constants from observed `branch:` values. | Reduces typos while keeping `branch:` easy to read. | Do not add `@AppBranch`. |
| Shell inheritance can be invisible. | Add `dust doctor` route table showing effective shell and branch. | Humans can audit route layout without reading generated code. | Do not require child routes to repeat `shell:`. |
| Bottom tab restore rules need examples. | Add a runnable tab-shell example page in `benchmark_project` or `shopping_app`. | Proves branch stacks in a real app. | Do not create separate shell marker routes. |
| Nested shell plus branch can be hard to debug. | Extend debug logs with `route`, `effectiveShell`, `effectiveBranch`, and `stackBefore/After`. | One trace explains the result. | Do not expose internal runtime classes as public API. |
| Some apps need independent nested Navigators. | Provide an escape hatch through a normal shell widget containing its own `Navigator` only for non-URL local flows. | Keeps advanced local flow possible. | Do not make this the default tab pattern. |

## URL Parameter And Route Result Improvements

| Feature | Current support | Suggested improvement | Example |
| :--- | :--- | :--- | :--- |
| Primitive path/query values | `String`, `int`, `double`, `bool`, nullable variants. | Keep as the default contract. | `/products/:id` -> `int id`. |
| Enum query values | Not first-class. | Add generated enum codecs for query values. | `?tab=reviews` -> `ProductTab.reviews`. |
| Date and time values | Not first-class. | Add opt-in `DateTime` ISO-8601 query parsing. | `?from=2026-08-10`. |
| URI values | Not first-class. | Support encoded `Uri` query values when constructor type is `Uri`. | `?redirect=https%3A%2F%2Fexample.com`. |
| Repeated query values | Not first-class. | Support `List<String>` and `List<int>` query parameters. | `?tag=a&tag=b`. |
| Route result typing | Supported through `result: Type`. | Add docs for typed route result failure cases and removed routes. | `await picker().push()` returns `bool?`. |

## Guard, Redirect, And Error Improvements

| Case | Suggested improvement | Acceptance test |
| :--- | :--- | :--- |
| Redirect loop | Include the route chain in `StateError`. | Error prints `login -> dashboard -> login`. |
| Guard failure | Log guard type, route, and redirect target. | Debug trace names the failing guard. |
| Async guard completion after route changed | Keep current stale-completion protection and document it. | Late async result cannot commit a stale route. |
| Auth loading state | Recommend returning `null` while auth is unresolved. | Protected route waits rather than bouncing. |
| Untrusted redirect path | Add a safe redirect parser recipe. | External redirect path cannot navigate inside app unexpectedly. |
| Unawaited navigation exception | Keep routing through `RouterBase.onException`. | Test captures thrown async guard error. |

## Generated Tooling Improvements

| Tooling | Suggested output | Human benefit |
| :--- | :--- | :--- |
| `dust doctor` route table | Route name, path, page, shell, branch, guards, result type. | One command verifies the route map. |
| Route graph export | Markdown or DOT graph of parent paths and shells. | Easier architecture review. |
| Deep-link fixture generator | Table of sample valid and invalid URLs from route annotations. | Easier QA and browser testing. |
| Route collision report | Static/dynamic sibling conflicts with examples. | Faster fixes for ambiguous routes. |
| Generated file header summary | Count routes, guarded routes, shells, branches. | Quick sanity check during review. |

## Test Suite Roadmap

| Suite | Cases to cover |
| :--- | :--- |
| Parser snapshots | Path params, query params, fragments, unknown query preservation, invalid typed params, percent encoding, trailing slash policy. |
| Browser history | Initial URL, refresh, back, forward, replace, same-location navigation, query-only navigation, guarded redirect history. |
| Deep links | Android app link shape, iOS universal link shape, web URL shape, cold start, warm link, unsafe host normalization. |
| Branch stacks | Direct branch deep link, branch switch, back across branch switch, child route inheriting nearest branch, non-branch route interaction. |
| Shells | Local shell, imported shell, hidden import, missing `Widget child`, nested shell override, shell plus branch. |
| Diagnostics | Parse failure, redirect decision, guard decision, branch restore, shell selection, async exception. |
| Generated output | Full-source snapshots for route table, parser, restore stack, navigation helpers, shell wrappers, branch constants. |

## Suggested Implementation Order

| Step | Work | Reason |
| :--- | :--- | :--- |
| 1 | Finish #407 web-history and deep-link tests for current behavior. | Prevent regressions before API changes. |
| 2 | Add `normalizeIncomingUri(Uri uri)` router override. | Covers unsafe hosts, prefixes, and subdirectory deploys without new annotations. |
| 3 | Add route diagnostics for parse failures and redirect chains. | Makes current behavior easier to debug. |
| 4 | Generate branch constants and route table metadata. | Reduces string mistakes while preserving simple `branch:` syntax. |
| 5 | Add enum, `DateTime`, `Uri`, and repeated query codecs. | Expands real web URL support. |
| 6 | Add hosting rewrite docs and a small web example checklist. | Makes path URL deployment practical. |
| 7 | Add route graph or `dust doctor` route table output. | Gives humans and agents a fast overview. |

## Non-Goals

| Non-goal | Reason |
| :--- | :--- |
| Add `@AppShellRoute` | Shells are already plain widgets through `shell: AppShell`; another annotation makes the API harder to remember. |
| Copy go_router `StatefulShellRoute` directly | Dust should keep the concept as `shell` for layout and `branch` for stack state. |
| Require app code to write `Navigator` for normal flows | Shareable flows should be normal typed routes; local `Navigator` remains only an advanced shell escape hatch. |
| Hide generated code completely | Generated route code is part of Dust's debugging and review story. |
| Support arbitrary object route parameters in URLs | URLs should stay explicit, stable, and serializable. Use IDs or small codecs. |

## Source Notes

- Flutter routing supports deep links and browser address-bar synchronization
  through Router for apps with specific deep-linking requirements:
  <https://docs.flutter.dev/ui/navigation>
- Flutter deep links open a specific location inside an app:
  <https://docs.flutter.dev/ui/navigation/deep-linking>
- Flutter web supports hash and path URL strategies:
  <https://docs.flutter.dev/ui/navigation/url-strategies>
- go_router is Navigation 2 based and supports deep linking and data-driven
  routes:
  <https://pub.dev/documentation/go_router/latest/go_router>
- AutoRoute documents strongly typed argument passing and deep linking:
  <https://pub.dev/packages/auto_route>
- Beamer is built around Flutter Router and Navigator pages API:
  <https://pub.dev/packages/beamer>

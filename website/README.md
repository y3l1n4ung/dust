# Dust website

Public website for [Dust](https://github.com/y3l1n4ung/dust), the Rust-powered
code generation engine for Dart and Flutter.

## Development

Requires Node.js 22.13 or newer.

```bash
npm install
npm run dev
```

The site runs at `http://localhost:3000`.

## Validation

```bash
npm test
```

This builds the Cloudflare Worker-compatible output and verifies the rendered
homepage, repository copy, links, and required branding assets.

## Artwork

The site uses the exact Happy Ferris SVG from
[Rustacean.net](https://rustacean.net/). Karen Rustad Tölva has waived copyright
and related rights to Ferris under CC0. The mascot composition places the
official Flutter and Dart marks in Ferris's raised claws.

Flutter and the related logo are trademarks of Google LLC. Dart and the related
logo are trademarks of Google LLC. Dust is not endorsed by or affiliated with
Google LLC.

## Command evidence

The homepage and workflow use output captured from real Dust commands after
generating 5,000 benchmark source files:

```bash
cd examples/benchmark_project
dart run tool/generate.dart --count 5000

cd ../..
target/release/dust build --root examples/benchmark_project
target/release/dust build --root examples/benchmark_project
target/release/dust check --root examples/product_showcase
```

The first run generated 5,010 outputs in 991 ms. The immediate warm run reused
5,007 cached outputs and finished in 251 ms. The product showcase check scanned
25 files with no stale output in 30 ms. These figures are a captured local run,
not a universal performance guarantee.

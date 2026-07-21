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
homepage, repository copy, links, and required social assets.

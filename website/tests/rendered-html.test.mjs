import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import test from "node:test";

const root = new URL("../", import.meta.url);

async function render() {
  const workerUrl = new URL("../dist/server/index.js", import.meta.url);
  workerUrl.searchParams.set("test", String(Date.now()));
  const { default: worker } = await import(workerUrl.href);

  return worker.fetch(
    new Request("https://dust.example/", {
      headers: { accept: "text/html", host: "dust.example" },
    }),
    { ASSETS: { fetch: async () => new Response("Not found", { status: 404 }) } },
    { waitUntil() {}, passThroughOnException() {} },
  );
}

test("server-renders the Dust project homepage", async () => {
  const response = await render();
  assert.equal(response.status, 200);
  assert.match(response.headers.get("content-type") ?? "", /^text\/html\b/i);

  const html = await response.text();
  assert.match(html, /<title>Dust — Built to make developers and AI agents happy<\/title>/i);
  assert.match(html, /Built to make developers and/);
  assert.match(html, /AI agents happy\./);
  assert.match(html, /Dust is Rust-powered code generation for Dart and Flutter/);
  assert.match(html, /https:\/\/github\.com\/y3l1n4ung\/dust/);
  assert.match(html, /dust-mascot\.png/);
  assert.doesNotMatch(html, /codex-preview|SkeletonPreview|react-loading-skeleton/);
});

test("keeps project copy and social assets intentional", async () => {
  const [page, layout, packageJson] = await Promise.all([
    readFile(new URL("app/page.tsx", root), "utf8"),
    readFile(new URL("app/layout.tsx", root), "utf8"),
    readFile(new URL("package.json", root), "utf8"),
    access(new URL("public/dust-mascot.png", root)),
    access(new URL("public/og.png", root)),
  ]);

  const description =
    "Dust is Rust-powered code generation for Dart and Flutter, built for human developers and AI coding agents. It handles repetitive and complex code so you can focus on your product.";

  assert.match(page, /Built to make developers and/);
  assert.ok(page.includes(description));
  assert.ok(layout.includes(description));
  assert.doesNotMatch(packageJson, /react-loading-skeleton|drizzle/);
});

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import { test } from "node:test";

const script = (await readFile(new URL("../apps/desktop-tauri/src-tauri/src/commands/gemini_apps/browser_probe.js", import.meta.url), "utf8"))
  .replace("__CODEXBAR_REQUEST_KEY__", '"probe"');

async function probe(fetch, location = { origin: "https://gemini.google.com" }) {
  const window = {};
  location.assign = (url) => { window.callback = url; };
  runInNewContext(script, { window, location, fetch, AbortController, URLSearchParams, TextDecoder, setTimeout, clearTimeout });
  const deadline = Date.now() + 2000;
  while (window.probe.kind === "pending" && Date.now() < deadline) {
    await new Promise(resolve => setImmediate(resolve));
  }
  assert.notEqual(window.probe.kind, "pending");
  assert.equal(new URL(window.callback).search, "");
  return window.probe;
}

test("uses the browser's own authentication and returns quota without CSRF or page data", async () => {
  const calls = [];
  const result = await probe(async (path, options) => {
    calls.push([path, options]);
    return new Response(calls.length === 1 ? '{"SNlM0e":"secret-csrf","account":"private@example.test"}' : 'quota-only');
  });
  assert.equal(result.kind, "ok");
  assert.equal(result.body, "quota-only");
  assert.equal(calls.length, 2);
  assert.equal(calls[1][1].body.get("at"), "secret-csrf");
  for (const [path, options] of calls) {
    assert.ok(path.startsWith("/"));
    assert.equal(options.credentials, "same-origin");
    assert.equal(options.redirect, "error");
    assert.equal(options.headers?.Cookie, undefined);
  }
  assert.ok(!JSON.stringify(result).includes("secret-csrf"));
  assert.ok(!JSON.stringify(result).includes("private@example.test"));
});

test("an anonymous HTTP 200 is authentication failure and never invokes quota", async () => {
  let count = 0;
  const result = await probe(async () => { count++; return new Response("<html>Sign in</html>"); });
  assert.equal(result.kind, "auth");
  assert.equal(count, 1);
  assert.equal(result.body, undefined);
});

test("an expired session does not poison a subsequent authenticated attempt", async () => {
  assert.equal((await probe(async () => new Response("expired", { status: 401 }))).kind, "auth");
  let count = 0;
  assert.equal((await probe(async () => new Response(++count === 1 ? '{"SNlM0e":"fresh"}' : 'quota'))).kind, "ok");
});

test("never returns oversized quota data", async () => {
  let count = 0;
  const result = await probe(async () => new Response(++count === 1 ? '{"SNlM0e":"csrf"}' : "x".repeat(512 * 1024 + 1)));
  assert.equal(result.kind, "too-large");
  assert.equal(result.body, undefined);
});

test("rejects navigation away from Gemini before exposing a response", async () => {
  const location = { origin: "https://gemini.google.com" };
  let count = 0;
  const result = await probe(async () => {
    if (++count === 1) return new Response('{"SNlM0e":"csrf"}');
    location.origin = "https://accounts.google.com";
    return new Response("must-not-return");
  }, location);
  assert.equal(result.kind, "auth");
  assert.equal(result.body, undefined);
});

test("network diagnostics do not export request errors or credentials", async () => {
  const result = await probe(async () => { throw new Error("Cookie=private-token"); });
  assert.equal(result.kind, "network");
  assert.ok(!JSON.stringify(result).includes("private-token"));
});

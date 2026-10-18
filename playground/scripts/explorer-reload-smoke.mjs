// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { createServer } from "node:http";

const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257);
const server = createServer((request, response) => {
  response.setHeader("Cache-Control", "no-store");
  if (request.url === "/echo") {
    response.end(request.headers.cookie || "");
    return;
  }
  response.setHeader("Content-Type", "text/html");
  response.end(`<!doctype html><title>Explorer reload fixture</title><script>
    document.cookie = 'explorer_reload=present;path=/;SameSite=Lax';
    addEventListener('message', event => {
      if (event.data?.probe) event.source.postMessage({ack: event.data.probe}, '*');
      if (event.data?.ack) window.parentAck ||= event.source === parent;
    });
    parent.postMessage({probe: 'loaded'}, '*');
    top.postMessage({probe: 'top-loaded'}, '*');
    window.cookieEcho = fetch('/echo', {credentials: 'include'}).then(r => r.text());
  </script><iframe srcdoc="<script>parent.postMessage({probe:'iframe'},'*')<\/script>"></iframe>`);
});
await new Promise(resolve => server.listen(Number(process.env.TUFFITE_EXPLORER_RELOAD_HTTP_PORT || 0), "::", resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const other = `http://localhost:${server.address().port}`;
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
browser.contexts().forEach(context => context.setDefaultTimeout(20000));
const app = browser.contexts().flatMap(context => context.pages()).find(page =>
  /\/(index\.html|main)$/.test(new URL(page.url()).pathname));
assert.ok(app, "Application main window is missing");

async function wait(probe, label) {
  const end = Date.now() + 20000;
  while (Date.now() < end) {
    const result = await probe();
    if (result) return result;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  throw Error(`Timed out: ${label}`);
}

// Inspect the page world: Explorer.executeJavaScript intentionally uses an
// isolated world and cannot inspect values set by the fixture's startup script.
async function evaluate(url, expression) {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  const target = targets.find(target => target.url === url);
  if (!target) return null;
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  try {
    return await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(Error(`Fixture evaluation timed out: ${url}: ${expression.slice(0, 160)}`)), 15000);
      socket.addEventListener("message", event => {
        const data = JSON.parse(event.data);
        if (data.id !== 1) return;
        clearTimeout(timer);
        if (data.error || data.result?.exceptionDetails) {
          reject(Error(JSON.stringify(data)));
        } else {
          resolve(data.result?.result?.value);
        }
      });
      socket.send(JSON.stringify({
        id: 1,
        method: "Runtime.evaluate",
        params: { expression, awaitPromise: true, returnByValue: true },
      }));
    });
  } finally {
    socket.close();
  }
}

try {
  await app.evaluate(async url => {
    window.reloadMessages = [];
    addEventListener("message", event => {
      if (event.data?.probe) {
        reloadMessages.push(event.data.probe);
        event.source.postMessage({ ack: event.data.probe }, "*");
      }
    });
    const owner = document.createElement("explorer");
    owner.navigation = "allow";
    owner.src = url;
    owner.style.cssText = "display:block;width:800px;height:500px";
    window.reloadOwner = owner;
    const loaded = new Promise(resolve => owner.addEventListener("loadstop", resolve, { once: true }));
    document.body.append(owner);
    await loaded;
  }, `${base}/owner`);
  console.log("explorer-reload phase=outer-loaded");
  await evaluate(`${base}/owner`, `(async () => {
    const child = document.createElement('explorer');
    child.navigation = 'allow';
    child.src = ${JSON.stringify(`${other}/child`)};
    window.reloadChild = child;
    window.childEvents = [];
    for (const type of ['loadstart', 'loadstop', 'error']) child.addEventListener(type, () => childEvents.push(type));
    document.body.append(child);
    return true;
  })()`);
  await wait(() => evaluate(`${other}/child`, `document.readyState === 'complete'`), "initial nested document");
  console.log("explorer-reload phase=child-loaded");
  for (let iteration = 0; iteration < 8; iteration++) {
    const url = iteration % 2 ? `${base}/child` : `${other}/child`;
    await evaluate(`${base}/owner`, `reloadChild.src = ${JSON.stringify(url)}; true`);
    await wait(() => evaluate(url, `document.readyState === 'complete'`), "cross-site child");
    const cookie = await evaluate(url, "cookieEcho");
    await evaluate(url, `
      parent.postMessage({probe: 'before-reload'}, '*');
      top.postMessage({probe: 'before-reload-top'}, '*'); true`);
    const before = await evaluate(`${base}/owner`, "reloadChild.getNavigationState()");
    await evaluate(`${base}/owner`, "reloadChild.reload({bypassCache: true})");
    await wait(async () => {
      const state = await evaluate(`${base}/owner`, "reloadChild.getNavigationState()");
      return state && !state.isLoading && state.documentId !== before.documentId;
    }, "reloaded child document");
    await wait(() => evaluate(url, `document.readyState === 'complete' && parentAck`), "reloaded child messaging");
    assert.deepEqual(await evaluate(url, `({parentSelf: parent === self, topSelf: top === self, nested: parent !== top})`), {
      parentSelf: false, topSelf: false, nested: true,
    });
    assert.match(await evaluate(url, "document.cookie"), /explorer_reload=present/);
    assert.match(cookie, /explorer_reload=present/);
    console.log(`explorer-reload iteration=${iteration} cookies=ok`);
  }
  const transferred = await evaluate(`${base}/owner`, `(async () => {
    const before = await reloadChild.getNavigationState();
    const handle = await reloadChild.detach();
    await reloadChild.attach(handle);
    const after = await reloadChild.getNavigationState();
    return {preserved: before.documentId === after.documentId, state: handle.state};
  })()`);
  assert.deepEqual(transferred, { preserved: true, state: "transferred" });
  const currentUrl = `${base}/child`;
  await evaluate(currentUrl, `parentAck = false; parent.postMessage({probe: 'reattached'}, '*'); true`);
  await wait(() => evaluate(currentUrl, "parentAck"), "reattached source-window reply");
  assert.ok(await app.evaluate(() => reloadMessages.includes("before-reload-top")),
    "Nested Explorer messages must reach the application top window");
  // Also reload the outer Explorer, as workbench pages usually have one level.
  for (let iteration = 0; iteration < 2; iteration++) {
    const before = await app.evaluate(() => reloadOwner.getNavigationState());
    await app.evaluate(() => reloadOwner.reload({ bypassCache: true }));
    await wait(async () => {
      const state = await app.evaluate(() => reloadOwner.getNavigationState());
      return !state.isLoading && state.documentId !== before.documentId;
    }, "reloaded outer document");
    await wait(() => evaluate(`${base}/owner`, "parentAck"), "outer messaging");
    assert.match(await evaluate(`${base}/owner`, "cookieEcho"), /explorer_reload=present/);
  }
  console.log(JSON.stringify({ crossSiteReload: true, outerReload: true, nested: true, detachAttach: true, cookies: true, messaging: true }));
} finally {
  await app.evaluate(() => window.reloadOwner?.remove()).catch(() => {});
  await browser.close();
  await new Promise(resolve => server.close(resolve));
}

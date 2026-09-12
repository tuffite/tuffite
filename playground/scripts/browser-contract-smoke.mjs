// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

import assert from "node:assert/strict";
import http from "node:http";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";

const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");

const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT);
const endpoint = `http://127.0.0.1:${port}`;
const requests = [];
let cacheRequests = 0;
const server = http.createServer((request, response) => {
  requests.push(request.headers);
  if (request.url === "/cache") {
    cacheRequests++;
    response.setHeader("Cache-Control", "public, max-age=3600");
    response.end("cached fixture");
  } else {
    response.setHeader("Cache-Control", "no-store");
    response.setHeader("Content-Type", "text/html");
    response.end("<!doctype html><title>Browser contract fixture</title><body>Local fixture</body>");
  }
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const fixture = `http://127.0.0.1:${server.address().port}`;
let browser;
let replacement;
async function until(check) {
  const deadline = Date.now() + 15000;
  do {
    const value = await check().catch(() => null);
    if (value) return value;
    await new Promise(resolve => setTimeout(resolve, 100));
  } while (Date.now() < deadline);
  throw Error("Browser contract fixture timed out");
}
async function connect() {
  console.log("browser-contract: connect");
  await until(async () => (await fetch(`${endpoint}/json/version`)).ok);
  browser = await chromium.connectOverCDP(endpoint);
  return until(async () => browser.contexts().flatMap(context => context.pages()).find(page => page.url().includes("index.html")));
}
async function explorer(root, partition) {
  const path = `/partition-${partition.startsWith("persist:") ? "persistent" : "memory"}`;
  await root.evaluate(({ partition, url }) => {
    const element = document.createElement("explorer");
    element.partition = partition;
    element.navigation = "allow";
    element.style.cssText = "display:block;width:400px;height:250px";
    element.src = url;
    document.body.append(element);
  }, { partition, url: fixture + path });
  return until(async () => browser.contexts().flatMap(context => context.pages()).find(page => page.url() === fixture + path));
}
async function closeBrowser(root) {
  // Playground also opens a configured secondary window. Close it before the
  // primary window so the application can exit through its normal quit path.
  for (const page of browser.contexts().flatMap(context => context.pages())) {
    if (page.url().endsWith("/window.html")) await page.close();
  }
  const session = await root.context().newCDPSession(root);
  const { targetInfo } = await session.send("Target.getTargetInfo");
  await session.send("Target.closeTarget", { targetId: targetInfo.targetId });
  await until(async () => {
    try { await fetch(`${endpoint}/json/version`); return false; }
    catch { return true; }
  });
}
try {
  let root = await connect();
  const version = await (await fetch(`${endpoint}/json/version`)).json();
  assert.match(version.Browser, /^Tuffite\/\d+/);
  console.log("browser-contract: persistent partition");
  const persistent = await explorer(root, "persist:browser-contract");
  console.log("browser-contract: memory partition");
  const memory = await explorer(root, "memory:browser-contract");
  await persistent.evaluate(() => document.cookie = "persistent=fixture;Max-Age=3600;SameSite=Lax");
  await memory.evaluate(() => document.cookie = "memory=fixture;Max-Age=3600;SameSite=Lax");
  assert.equal(await persistent.evaluate(() => document.cookie), "persistent=fixture");
  assert.equal(await memory.evaluate(() => document.cookie), "memory=fixture");
  console.log("browser-contract: root navigation");
  await root.goto(fixture);
  assert.deepEqual(await root.evaluate(() => navigator.languages), ["en-US", "en"]);
  assert.deepEqual(await root.evaluate(() => new Promise(resolve => {
    const blob = new Blob(["postMessage(navigator.languages)"], { type: "text/javascript" });
    const url = URL.createObjectURL(blob);
    const worker = new Worker(url);
    worker.onmessage = event => { worker.terminate(); URL.revokeObjectURL(url); resolve(event.data); };
  })), ["en-US", "en"]);
  const session = await root.context().newCDPSession(root);
  await session.send("Page.enable");
  for (const [expression, accept, promptText, expected] of [
    ["alert('fixture')", true, "", undefined],
    ["confirm('fixture')", false, "", false],
    ["prompt('fixture','initial')", true, "edited", "edited"],
  ]) {
    const opening = new Promise(resolve => session.once("Page.javascriptDialogOpening", resolve));
    // Preserve dialogs instead of Playwright's default auto-dismiss.
    const preserve = () => {};
    root.on("dialog", preserve);
    const result = root.evaluate(expression);
    assert.equal((await opening).hasBrowserHandler, true);
    await session.send("Page.handleJavaScriptDialog", { accept, promptText });
    assert.equal(await result, expected);
    root.off("dialog", preserve);
  }
  root.on("dialog", () => {});
  await root.locator("body").click();
  await root.evaluate(() => addEventListener("beforeunload", event => {
    event.preventDefault();
    event.returnValue = "";
  }));
  const beforeUnload = new Promise(resolve => session.once("Page.javascriptDialogOpening", resolve));
  const navigation = root.goto(fixture + "/next");
  assert.equal((await beforeUnload).hasBrowserHandler, true);
  await session.send("Page.handleJavaScriptDialog", { accept: true });
  await navigation;
  await root.evaluate(() => {
    document.cookie = "root=fixture;Max-Age=3600;SameSite=Lax";
    document.cookie = "session=fixture;SameSite=Lax";
  });
  await root.evaluate(() => fetch("/cache").then(response => response.text()));
  assert.equal(cacheRequests, 1);
  // Close all native windows so their network contexts commit pending cookie
  // operations through the normal browser shutdown path.
  console.log("browser-contract: normal shutdown");
  await closeBrowser(root);
  const command = JSON.parse(process.env.TUFFITE_PLAYGROUND_COMMAND);
  replacement = spawn(command[0], command.slice(1), {
    cwd: process.env.TUFFITE_PLAYGROUND_CWD,
    stdio: "ignore",
  });
  root = await connect();
  const restartedPersistent = await explorer(root, "persist:browser-contract");
  const restartedMemory = await explorer(root, "memory:browser-contract");
  assert.equal(await restartedPersistent.evaluate(() => document.cookie), "persistent=fixture");
  assert.equal(await restartedMemory.evaluate(() => document.cookie), "");
  console.log("browser-contract: root navigation");
  await root.goto(fixture);
  assert.equal(await root.evaluate(() => document.cookie), "root=fixture");
  assert.equal(await root.evaluate(() => fetch("/cache").then(response => response.text())), "cached fixture");
  assert.equal(cacheRequests, 1, "HTTP cache should survive process restart");
  assert.ok(requests.every(headers => headers["accept-language"] === "en-US,en;q=0.9"));
  console.log("Browser contracts passed: product, page/worker/network languages, native dialog handlers, encrypted cookie restart, session/memory isolation, HTTP cache restart");
  console.log("browser-contract: normal shutdown");
  await closeBrowser(root);
} finally {
  replacement?.kill();
  await browser?.close().catch(() => {});
  server.close();
}

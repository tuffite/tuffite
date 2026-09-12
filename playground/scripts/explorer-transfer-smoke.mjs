import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT ?? "9257");
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
const ownDialogs = (page) => page.on("dialog", () => {});
for (const context of browser.contexts()) {
  context.on("page", ownDialogs);
  context.pages().forEach(ownDialogs);
}
const pages = () => browser.contexts().flatMap((context) => context.pages());
let app;
await until(() => (app = pages().find((page) => page.url().includes("/index.html"))), "Playground main window is missing");
const base = new URL("./", app.url()).href;
const targetCount = async () => {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  return targets.filter((target) => /(?:history-[ab]|transfer-sample)\.html/.test(target.url)).length;
};
async function until(check, message) {
  const end = Date.now() + 15000;
  while (Date.now() < end) {
    if (await check()) return;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(message);
}
try {
  await app.goto(new URL("transfer.html", base).href);
  await app.waitForFunction(() => window.transferDemo && typeof window.transferDemo.first.detach === "function");
  const initial = await app.evaluate(() => window.transferDemo.load());
  assert.ok(initial.state.documentId);
  const liveRoot = pages().find((page) => page.url().includes("transfer-sample.html"));
  assert.ok(liveRoot);
  const nestedBefore = await liveRoot.evaluate(async () => {
    const nested = document.createElement("explorer");
    nested.id = "nested-transfer";
    nested.navigation = "allow";
    nested.style.cssText = "display:block;width:400px;height:100px";
    const loaded = new Promise((resolve) => nested.addEventListener("loadstop", resolve, { once: true }));
    nested.src = new URL("./history-b.html", location.href).href;
    document.body.append(nested);
    await loaded;
    await nested.executeJavaScript("globalThis.nestedMarker = 'nested-unsaved'");
    return nested.getNavigationState();
  });
  const baseline = await targetCount();
  assert.equal(baseline, 2);
  await app.evaluate(() => {
    const deferred = document.createElement("explorer");
    deferred.deferred = true;
    deferred.navigation = "allow";
    deferred.src = new URL("./history-a.html?deferred-check", document.baseURI).href;
    document.body.append(deferred);
    window.deferredCheck = deferred;
  });
  assert.equal(await targetCount(), baseline, "A deferred host must not create a page");
  const configured = await app.evaluate(async () => {
    const element = window.deferredCheck;
    const loaded = new Promise((resolve) => element.addEventListener("loadstop", resolve, { once: true }));
    element.deferred = false;
    await loaded;
    const state = await element.getNavigationState();
    element.remove();
    return state;
  });
  assert.ok(configured.url.includes("history-a.html?deferred-check"));
  await until(async () => await targetCount() === baseline, "Deferred test retained an extra page");

  // The standard same-document move must retain the element and its page.
  const move = await app.evaluate(async () => {
    const demo = window.transferDemo;
    const holder = document.createElement("div");
    document.body.append(holder);
    holder.moveBefore(demo.active, null);
    const after = await demo.snapshot();
    document.querySelector("#hosts").moveBefore(demo.active, null);
    holder.remove();
    return after;
  });
  assert.equal(move.state.documentId, initial.state.documentId);
  assert.equal(move.marker, initial.marker);

  const started = Date.now();
  const stress = await app.evaluate(async () => {
    const demo = window.transferDemo;
    const before = await demo.snapshot();
    const loads = demo.events.filter((event) => event.startsWith("load")).length;
    for (let i = 0; i < 25; ++i) { await demo.detach(); await demo.attach(); }
    return { before, after: await demo.snapshot(), loads, afterLoads: demo.events.filter((event) => event.startsWith("load")).length };
  });
  assert.equal(stress.before.state.documentId, stress.after.state.documentId);
  assert.equal(stress.before.state.rendererProcess.id, stress.after.state.rendererProcess.id);
  assert.equal(stress.before.marker, stress.after.marker);
  assert.equal(stress.before.draft, stress.after.draft);
  assert.equal(stress.before.historyLength, stress.after.historyLength);
  assert.equal(stress.loads, stress.afterLoads, "Transfers must not emit load events");
  assert.equal(await targetCount(), baseline, "Repeated transfers must not retain extra pages");
  const nestedAfter = await liveRoot.evaluate(async () => {
    const nested = document.querySelector("#nested-transfer");
    return { state: await nested.getNavigationState(), marker: await nested.executeJavaScript("globalThis.nestedMarker") };
  });
  assert.equal(nestedAfter.state.documentId, nestedBefore.documentId);
  assert.equal(nestedAfter.marker, "nested-unsaved");
  const stressMs = Date.now() - started;
  // The replacement host must also own the page's dialog delegate.
  const dialog = await app.evaluate(async () => {
    const element = window.transferDemo.active;
    let seen = false;
    element.addEventListener("dialog", (event) => {
      seen = true;
      event.stopPropagation();
      void event.detail.dismiss();
    }, { once: true });
    const value = await element.executeJavaScript("confirm('after transfer')");
    return { seen, value };
  });
  assert.deepEqual(dialog, { seen: true, value: false });

  // Reject a mismatched policy without consuming the credential, then recover.
  const rejected = await app.evaluate(async () => {
    const demo = window.transferDemo;
    const token = await demo.detach();
    const wrong = document.createElement("explorer");
    wrong.deferred = true;
    wrong.navigation = "emit";
    document.body.append(wrong);
    let error;
    try { await wrong.attach(token); } catch (failure) { error = failure.name; }
    wrong.remove();
    const after = await demo.attach();
    wrong.navigation = "allow";
    document.body.append(wrong);
    let duplicate;
    try { await wrong.attach(token); } catch (failure) { duplicate = failure.name; }
    wrong.remove();
    return { error, duplicate, after };
  });
  assert.equal(rejected.error, "SecurityError");
  assert.equal(rejected.duplicate, "InvalidStateError");
  assert.equal(rejected.after.marker, initial.marker);

  // Removing the empty source and GC of the wrapper must not release its page.
  const pending = await app.evaluate(async () => {
    const source = window.transferDemo.active;
    let handle = await source.detach();
    window.pendingToken = handle.token;
    source.remove();
    handle = undefined;
    return window.pendingToken;
  });
  const session = await app.context().newCDPSession(app);
  await session.send("HeapProfiler.collectGarbage");
  await session.detach();
  assert.equal(await targetCount(), baseline);
  const retained = await app.evaluate(async (token) => {
    const target = document.createElement("explorer");
    target.deferred = true;
    target.navigation = "allow";
    target.dialogs = "emit";
    document.querySelector("#hosts").append(target);
    await target.attach(token);
    const state = await target.getNavigationState();
    const marker = await target.executeJavaScript("globalThis.transferMarker");
    const handle = await target.detach();
    await handle.close();
    await handle.close(); // idempotent
    target.remove();
    return { state, marker, closed: handle.state };
  }, pending);
  assert.equal(retained.state.documentId, initial.state.documentId);
  assert.equal(retained.marker, initial.marker);
  assert.equal(retained.closed, "closed");
  await until(async () => await targetCount() === 0, "close() did not destroy the pending page");

  // Cross-window transfer survives the original document's replacement.
  await app.goto(new URL("transfer.html", base).href);
  await app.waitForFunction(() => window.transferDemo);
  await app.evaluate(() => window.transferDemo.load());
  const independent = await app.evaluate(() => window.transferDemo.popout());
  assert.equal(independent.before.state.documentId, independent.after.state.documentId);
  assert.equal(independent.before.state.rendererProcess.id, independent.after.state.rendererProcess.id);
  assert.equal(independent.before.marker, independent.after.marker);
  assert.equal(independent.before.draft, independent.after.draft);
  assert.equal(independent.before.historyLength, independent.after.historyLength);
  await app.goto(new URL("window.html", base).href);
  const child = pages().find((page) => page.url().includes("transfer.html?receive="));
  assert.ok(child, "Independent target window is missing");
  const live = await child.evaluate(() => window.transferDemo.snapshot());
  assert.equal(live.marker, independent.before.marker);
  assert.equal(await targetCount(), 1);
  await child.evaluate(() => window.tuffite.window.close());
  await until(async () => await targetCount() === 0, "Target window close retained a page");

  // Source document teardown cleans an abandoned credential.
  await app.goto(new URL("transfer.html", base).href);
  await app.waitForFunction(() => window.transferDemo);
  await app.evaluate(() => window.transferDemo.load());
  const abandoned = await app.evaluate(() => window.transferDemo.detach());
  await app.goto(new URL("transfer.html?fresh", base).href);
  await app.waitForFunction(() => window.transferDemo);
  await until(async () => await targetCount() === 0, "Source document teardown retained a page");
  const expired = await app.evaluate(async (token) => {
    try { await window.transferDemo.first.attach(token); return "accepted"; }
    catch (error) { return error.name; }
  }, abandoned);
  assert.equal(expired, "InvalidStateError");
  // An ordinary iframe can also be the owner document.
  const iframeUrl = new URL("transfer.html?iframe-owner", base).href;
  await app.evaluate(async (url) => {
    const frame = document.createElement("iframe");
    frame.id = "transfer-owner-frame";
    // The App scheme uses opaque origins. srcdoc inherits the parent's origin,
    // allowing the parent to retain the handle after this document navigates.
    frame.srcdoc = (await (await fetch(url)).text()).replace(
      "<head>", `<head><base href="${url}">`,
    );
    document.body.append(frame);
  }, iframeUrl);
  await until(() => app.frames().some((frame) => frame.url() === "about:srcdoc"), "Iframe owner missing");
  const iframeOwner = app.frames().find((frame) => frame.url() === "about:srcdoc");
  await iframeOwner.waitForFunction(() => window.transferDemo);
  await iframeOwner.evaluate(() => window.transferDemo.load());
  const iframeToken = await app.evaluate(async () => {
    const source = document.querySelector("#transfer-owner-frame").contentWindow.transferDemo.active;
    window.iframeTransfer = await source.detach();
    return window.iframeTransfer.token;
  });
  await iframeOwner.goto(new URL("window.html", base).href);
  await until(async () => await targetCount() === 0, "Iframe owner navigation retained a pending page");
  assert.equal(await app.evaluate(() => window.iframeTransfer.state), "closed");
  const iframeState = await app.evaluate(async (token) => {
    try { await window.transferDemo.first.attach(token); return "accepted"; }
    catch (error) { return error.name; }
  }, iframeToken);
  assert.equal(iframeState, "InvalidStateError");
  await app.evaluate(() => document.querySelector("#transfer-owner-frame").remove());
  const sourceWindow = await app.evaluate(async (url) => {
    const child = await window.tuffite.window.create({
      url, title: "Pending transfer owner", bounds: { x: 120, y: 120, width: 800, height: 600 },
    });
    return child.id.toString();
  }, new URL("transfer.html?source-window", base).href);
  await until(() => pages().some((page) => page.url().includes("?source-window")), "Source window missing");
  const owner = pages().find((page) => page.url().includes("?source-window"));
  await owner.waitForFunction(() => window.transferDemo);
  await owner.evaluate(() => window.transferDemo.load());
  const orphan = await owner.evaluate(() => window.transferDemo.detach());
  await app.evaluate((id) => window.tuffite.window.close(BigInt(id)), sourceWindow);
  await until(async () => await targetCount() === 0, "Source window close retained a pending page");
  const orphanState = await app.evaluate(async (token) => {
    try { await window.transferDemo.first.attach(token); return "accepted"; }
    catch (error) { return error.name; }
  }, orphan);
  assert.equal(orphanState, "InvalidStateError");
  // A renderer crash leaves its browser-side document object alive. The
  // document registry must still release its pending page immediately.
  await app.goto(new URL("transfer.html?crash-owner", base).href);
  await app.waitForFunction(() => window.transferDemo);
  await app.evaluate(() => window.transferDemo.load());
  await app.evaluate(() => window.transferDemo.detach());
  const crashSession = await app.context().newCDPSession(app);
  const crashed = app.waitForEvent("crash", { timeout: 10000 });
  void crashSession.send("Page.crash").catch(() => {});
  await crashed;
  await until(async () => await targetCount() === 0, "Source renderer crash retained a pending page");
  // A crashed renderer cannot acknowledge session teardown.
  void crashSession.detach().catch(() => {});
  console.log(JSON.stringify({ explorerTransfer: "ok", stressTransfers: 25, stressMs, meanTransferMs: stressMs / 25, remainingSamplePages: await targetCount() }));
} finally {
  await browser.close();
}

// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

import assert from "node:assert/strict";
import http from "node:http";
import { chromium } from "playwright-core";

const requests = [];
const server = http.createServer((request, response) => {
  requests.push({ url: request.url, host: request.headers.host, headers: request.headers });
  if (request.url === "/client-hints-opt-in") {
    response.setHeader("Accept-CH", "Sec-CH-UA-Arch, Sec-CH-UA-Full-Version-List");
  }
  response.setHeader("Content-Type", "text/html");
  if (request.url === "/input-frame") {
    response.end(`<!doctype html><style>body{margin:0}button{position:absolute;left:24px;top:24px;width:200px;height:48px}</style>
      <button id="input-fixture">Local input fixture</button><script>
      for (const type of ["pointermove", "pointerdown", "pointerup", "click", "keydown"]) {
        document.addEventListener(type, event => parent.postMessage({
          fixture: "input", type, target: event.target.tagName, trusted: event.isTrusted, focused: document.hasFocus(),
          visibility: document.visibilityState, active: navigator.userActivation.isActive,
          x: event.clientX, y: event.clientY, key: event.key
        }, "*"));
      }
      </script>`);
    return;
  }
  if (request.url === "/input-parent") {
    response.end(`<!doctype html><style>body{margin:0}iframe{position:absolute;left:120px;top:100px;width:360px;height:160px;border:0}</style>
      <iframe src="http://127.0.0.1:${server.address().port}/input-frame"></iframe>
      <script>window.inputEvents=[];addEventListener("message", event => {
        if(event.source===document.querySelector("iframe").contentWindow && event.data.fixture==="input")inputEvents.push(event.data);
      });</script>`);
    return;
  }
  response.end("<!doctype html><title>WebAuthn fixture</title><body>Passkey fixture</body>");
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const url = `http://localhost:${server.address().port}`;
const browser = await chromium.connectOverCDP(
  `http://127.0.0.1:${process.env.TUFFITE_PLAYGROUND_CDP_PORT ?? 9257}`,
);
const pages = () => browser.contexts().flatMap((context) => context.pages());
async function until(check, message) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const result = await check();
    if (result) return result;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(message);
}
try {
  const app = await until(
    () => pages().find((page) => page.url().includes("/index.html")),
    "Missing Playground",
  );
  const mountExplorer = (url) => app.evaluate(async (url) => {
    window.webauthnExplorer?.remove();
    const explorer = document.createElement("explorer");
    explorer.id = "webauthn-fixture-explorer";
    explorer.navigation = "allow";
    explorer.style.cssText = "position:fixed;inset:0;display:block;z-index:9999";
    explorer.src = url;
    window.webauthnExplorer = explorer;
    const ready = new Promise((resolve) => explorer.addEventListener("loadstop", resolve, { once: true }));
    document.body.append(explorer);
    await ready;
  }, url);
  await mountExplorer(url);
  let page = await until(() => pages().find((page) => page.url().startsWith(url)), "Missing Explorer target");
  assert.deepEqual(await page.evaluate(() => {
    localStorage.setItem("explorer-session-check", "present");
    document.cookie = "explorer-session-check=present; Path=/; SameSite=Strict";
    const storage = localStorage.getItem("explorer-session-check") === "present";
    localStorage.removeItem("explorer-session-check");
    const cookies = document.cookie.includes("explorer-session-check=present");
    document.cookie = "explorer-session-check=; Path=/; Max-Age=0";
    return { topLevel: window.top === window, parentIsSelf: window.parent === window, storage, cookies };
  }), { topLevel: true, parentIsSelf: true, storage: true, cookies: true },
  "Explorer must expose an independent top-level page with working session storage");
  await page.goto(`${url}/input-parent`);
  await until(async () => (await page.frames().find(frame => frame.url().endsWith("/input-frame"))
    ?.locator("#input-fixture").count()) === 1, "Missing cross-origin input fixture");
  await app.bringToFront();
  const bounds = await app.locator("#webauthn-fixture-explorer").boundingBox();
  assert.ok(bounds, "Explorer must have clickable outer bounds");
  // Route through the outer app's compositor hit test, never directly to the
  // Explorer/iframe CDP target. This exercises the actual embedding boundary.
  await app.waitForTimeout(1000);
  await app.mouse.move(bounds.x + 244, bounds.y + 148);
  await app.mouse.click(bounds.x + 244, bounds.y + 148);
  await app.waitForTimeout(1000);
  await until(async () => (await page.evaluate(() => window.inputEvents))
    .some(event => event.type === "click"), "Outer input did not reach the Explorer iframe");
  const inputEvents = await page.evaluate(() => window.inputEvents);
  for (const type of ["pointerdown", "pointerup", "click"]) {
    const event = inputEvents.find(event => event.type === type);
    assert.ok(event, `Missing ${type} across Explorer and cross-origin frame`);
    assert.equal(event.trusted, true);
    assert.equal(event.visibility, "visible");
    assert.equal(event.active, true);
    assert.ok(Math.abs(event.x - 124) <= 1 && Math.abs(event.y - 48) <= 1,
      "Pointer coordinates must be relative to the nested frame");
  }
  await app.keyboard.press("Space");
  await until(async () => (await page.evaluate(() => window.inputEvents))
    .some(event => event.type === "keydown" && event.key === " "),
    "Keyboard focus did not follow the pointer into the Explorer iframe");
  assert.equal(await page.frames().find(frame => frame.url().endsWith("/input-frame")).evaluate(() => document.hasFocus()), true);
  const inputSession = await page.context().newCDPSession(page);
  const ancestorWrite = await inputSession.send("Runtime.evaluate", {
    expression: 'navigator.clipboard.writeText("unfocused ancestor").then(() => "granted", error => error.name)',
    awaitPromise: true, returnByValue: true, userGesture: false,
  });
  assert.equal(ancestorWrite.result.value, "NotAllowedError",
    "An ancestor of the explicitly focused iframe must not receive a clipboard grant");
  await inputSession.detach();
  console.log("Explorer input: outer hit testing, cross-origin pointer coordinates, focus, visibility and activation passed");
  const navigationHeaders = async (target) => {
    await page.goto(target);
    const parsed = new URL(target);
    const request = requests.findLast((request) =>
      request.url === parsed.pathname && request.host === parsed.host);
    assert.ok(request, "Fixture must observe a real navigation request");
    return request.headers;
  };
  const initialHints = await navigationHeaders(`${url}/client-hints-initial`);
  const metadata = await page.evaluate(async () => ({
    brands: navigator.userAgentData.brands,
    mobile: navigator.userAgentData.mobile,
    platform: navigator.userAgentData.platform,
    ...await navigator.userAgentData.getHighEntropyValues(["architecture", "fullVersionList"]),
  }));
  const serializeBrands = (brands) => brands.map(({ brand, version }) =>
    `${JSON.stringify(brand)};v=${JSON.stringify(version)}`).join(", ");
  assert.equal(initialHints["sec-ch-ua"], serializeBrands(metadata.brands));
  assert.equal(initialHints["sec-ch-ua-mobile"], metadata.mobile ? "?1" : "?0");
  assert.equal(initialHints["sec-ch-ua-platform"], JSON.stringify(metadata.platform));
  assert.equal(initialHints["sec-ch-ua-arch"], undefined,
    "High entropy hints must require an origin opt-in");
  await navigationHeaders(`${url}/client-hints-opt-in`);
  const optedInHints = await navigationHeaders(`${url}/client-hints-next`);
  assert.equal(optedInHints["sec-ch-ua-arch"], JSON.stringify(metadata.architecture));
  assert.equal(optedInHints["sec-ch-ua-full-version-list"], serializeBrands(metadata.fullVersionList));
  const isolatedHints = await navigationHeaders(
    `http://127.0.0.1:${server.address().port}/client-hints-isolated`);
  assert.equal(isolatedHints["sec-ch-ua-arch"], undefined,
    "Another origin must not inherit Client Hints opt-ins");
  assert.equal(isolatedHints["sec-ch-ua-full-version-list"], undefined);
  console.log("Explorer Client Hints: HTTP/JavaScript metadata match, Accept-CH opt-in and origin isolation passed");
  // Start a fresh document so earlier evaluation cannot leave user activation.
  await page.goto(`${url}?clipboard`);
  const session = await page.context().newCDPSession(page);
  const clipboardDenied = async (expression) => {
    const result = await session.send("Runtime.evaluate", {
      expression: `${expression}.then(() => "granted", error => error.name)`,
      awaitPromise: true,
      returnByValue: true,
      userGesture: false,
    });
    assert.equal(result.result.value, "NotAllowedError");
  };
  await page.bringToFront();
  await clipboardDenied('navigator.clipboard.writeText("unactivated write")');
  await page.evaluate(() => {
    const button = document.createElement("button");
    button.id = "clipboard-copy";
    button.textContent = "Copy fixture result";
    window.clipboardCopyResult = "pending";
    button.onclick = () => navigator.clipboard.writeText("Tuffite clipboard regression")
      .then(() => { window.clipboardCopyResult = "copied"; },
        error => { window.clipboardCopyResult = `${error.name}: ${error.message}`; });
    document.body.append(button);
  });
  const copyBounds = await page.locator("#clipboard-copy").boundingBox();
  assert.ok(copyBounds);
  await app.bringToFront();
  await app.waitForTimeout(1000);
  await app.mouse.click(bounds.x + copyBounds.x + copyBounds.width / 2,
    bounds.y + copyBounds.y + copyBounds.height / 2);
  await until(async () => (await page.evaluate(() => window.clipboardCopyResult)) !== "pending",
    "Clipboard write did not finish");
  assert.equal(await page.evaluate(() => window.clipboardCopyResult), "copied");
  await clipboardDenied("navigator.clipboard.readText()");
  console.log("Explorer clipboard: click-to-copy passed, unactivated writes and reads denied");
  await session.send("WebAuthn.enable");
  const { authenticatorId } = await session.send("WebAuthn.addVirtualAuthenticator", {
    options: {
      protocol: "ctap2",
      transport: "internal",
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
      automaticPresenceSimulation: true,
    },
  });
  assert.equal(await page.evaluate(() => PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable()), true);
  const registered = await page.evaluate(async () => {
    const credential = await navigator.credentials.create({ publicKey: {
      challenge: crypto.getRandomValues(new Uint8Array(32)),
      rp: { id: "localhost", name: "Local fixture" },
      user: { id: new Uint8Array([1]), name: "fixture", displayName: "Fixture account" },
      pubKeyCredParams: [{ type: "public-key", alg: -7 }, { type: "public-key", alg: -257 }],
      authenticatorSelection: { residentKey: "required", userVerification: "required" },
      timeout: 10000,
    } });
    return credential.id;
  });
  const asserted = await page.evaluate(async () => {
    const credential = await navigator.credentials.get({ publicKey: {
      challenge: crypto.getRandomValues(new Uint8Array(32)), rpId: "localhost",
      allowCredentials: [], userVerification: "required", timeout: 10000,
    } });
    return {
      id: credential.id,
      user: Array.from(new Uint8Array(credential.response.userHandle)),
      verified: Boolean(new Uint8Array(credential.response.authenticatorData)[32] & 0x04),
    };
  });
  assert.equal(asserted.id, registered);
  assert.deepEqual(asserted.user, [1]);
  assert.equal(asserted.verified, true);
  // An IP RP ID is rejected locally. A foreign domain triggers Chromium's
  // related-origin network validation and can race a short request timeout.
  const rejected = await page.evaluate(async () => {
    try {
      await navigator.credentials.get({ publicKey: {
        challenge: new Uint8Array(32), rpId: "127.0.0.1", timeout: 1000,
      } });
      return "accepted";
    } catch (error) { return error.name; }
  });
  assert.equal(rejected, "SecurityError", "Cross-origin RP access must remain blocked");
  await session.send("WebAuthn.removeVirtualAuthenticator", { authenticatorId });
  await session.send("WebAuthn.disable");
  // Chromium keeps DisableUI on the document's AuthenticatorCommonImpl after
  // a virtual request. A fresh document is required for real embedder UI.
  const realUrl = `${url}/real`;
  await mountExplorer(realUrl);
  page = await until(() => pages().find((page) => page.url() === realUrl), "Missing real Explorer target");
  // Exercise the real embedder UI's destruction through AbortSignal. No OS
  // credentials or automated system permission approvals are involved.
  for (const mediation of ["optional", "optional", "conditional"]) {
    const outcome = await page.evaluate(async (mediation) => {
      const started = Date.now();
      const abort = new AbortController();
      setTimeout(() => abort.abort(), 800);
      try {
        await navigator.credentials.get({ mediation, signal: abort.signal, publicKey: {
          challenge: crypto.getRandomValues(new Uint8Array(32)), rpId: "localhost",
          allowCredentials: [], timeout: 10000,
        } });
        return { name: "accepted" };
      } catch (error) {
        return { name: error.name, message: error.message, aborted: abort.signal.aborted, elapsed: Date.now() - started };
      }
    }, mediation);
    console.log(JSON.stringify({ mediation, ...outcome }));
    assert.equal(outcome.name, "AbortError", "Real request must cancel and allow another request");
  }
  console.log("Explorer WebAuthn: resident create/get, UV, RP isolation and repeated modal and conditional cancellation passed");
} finally {
  await browser.close();
  await new Promise((resolve) => server.close(resolve));
}

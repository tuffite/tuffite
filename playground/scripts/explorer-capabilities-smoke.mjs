import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");
const endpoint = `http://127.0.0.1:${process.env.TUFFITE_PLAYGROUND_CDP_PORT ?? 9257}`;
const browser = await chromium.connectOverCDP(endpoint);
const pages = () => browser.contexts().flatMap((context) => context.pages());
async function until(check, message) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    if (await check()) return;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(message);
}
try {
  let app;
  await until(() => (app = pages().find((page) => page.url().includes("/index.html"))), "Playground main window is missing");
  await app.goto(new URL("capabilities.html", app.url()).href);
  await app.waitForFunction(() => window.capabilitiesDemo);
  let artifact, preview;
  await until(() => {
    artifact = pages().find((page) => page.url().includes("demo.artifact"));
    preview = pages().find((page) => page.url().includes("demo.preview"));
    return artifact && preview;
  }, "Separate artifact/preview WebContents were not created");
  for (const page of [artifact, preview])
    await page.waitForFunction(() => document.body.dataset.capabilityReport);
  const report = async (page) => page.evaluate(() => JSON.parse(document.body.dataset.capabilityReport));
  const a = await report(artifact), p = await report(preview);
  assert.notEqual(a.origin, p.origin, "Sharing a capability must not merge origins");
  for (const result of [a, p]) {
    assert.equal(result.applicationGlobal, false);
    for (const field of ['typedUtility', 'deniedEndpoint', 'deniedService']) {
      assert.equal(result[field].allowed, false);
      assert.equal(result[field].status, 2, 'Origin must not gain a Utility endpoint from another origin');
    }
    assert.equal(result.typedCommand.allowed, false);
    assert.equal(result.typedCommand.status, 2, "Typed calls must be rejected by native authorization");
    assert.equal(result.read, "allowed");
    assert.match(result.write, /^denied:/);
    assert.doesNotMatch(result.write, /expected Uint8Array/, "Write denial must come from native authorization");
    assert.equal(result.createExplorer, false);
  }
  const fileScopes = await artifact.evaluate(async () => {
    const frameworkApi = await import('/tuffite.js');
    const allowed = await frameworkApi.fs.readFile(frameworkApi.BaseDirectory.Resource, "ui/dist/transfer-sample.html");
    const denied = async (path) => {
      try { await frameworkApi.fs.readFile(frameworkApi.BaseDirectory.Resource, path); return false; }
      catch (error) { return (error.status ?? error.cause?.status) === 2; }
    };
    return { allowed: allowed.length > 0, explicitDeny: await denied("ui/dist/index.html"), outsideScope: await denied("shell_api.js") };
  });
  assert.deepEqual(fileScopes, { allowed: true, explicitDeny: true, outsideScope: true });

  assert.equal(a.appCommand, "allowed", "A second matching capability must union its grants");
  assert.match(p.appCommand, /^denied:/);

  assert.equal(await app.evaluate(async () => (await fetch("/not-mapped.html")).status), 404, "Unmapped paths must not fall back to the SPA");
  await app.evaluate(() => {
    const host = document.createElement("explorer");
    host.src = "playground://public/ui/dist/transfer-sample.html";
    document.querySelector("#hosts").append(host);
    window.capabilitiesDemo.publicHost = host;
  });
  let publicPage;
  await until(() => (publicPage = pages().find((page) => page.url().includes("playground://public/"))), "Public asset-only origin missing");
  await publicPage.waitForFunction(() => document.querySelector("#history-state")?.textContent.includes("History length"));
  const publicReport = await publicPage.evaluate(() => ({bridge: typeof globalThis.tuffite?.invoke, createExplorer: typeof document.createElement("explorer").getNavigationState}));
  assert.equal(publicReport.bridge, "undefined", "Asset routing must not grant native APIs");
  assert.equal(publicReport.createExplorer, "undefined");
  await app.evaluate(() => window.capabilitiesDemo.publicHost.remove());
  await until(() => !pages().some((page) => page.url().includes("playground://public/")), "Public page leaked after host removal");

  await app.evaluate(() => {
    const host = document.createElement("explorer");
    host.src = "playground://business/ui/dist/capability-sample.html";
    document.querySelector("#hosts").append(host);
    window.capabilitiesDemo.businessHost = host;
  });
  let businessPage;
  await until(() => (businessPage = pages().find((page) => page.url().includes("playground://business/"))), "Business-only origin missing");
  await businessPage.waitForFunction(() => document.body.dataset.capabilityReport);
  const businessReport = await report(businessPage);
  assert.equal(businessReport.appCommand, "allowed");
  assert.equal(businessReport.applicationGlobal, false);
  assert.equal(businessReport.typedUtility.allowed, true, 'Explicit service/endpoint grant must reach Utility Rust');
  for (const field of ['deniedEndpoint', 'deniedService']) {
    assert.equal(businessReport[field].allowed, false);
    assert.equal(businessReport[field].status, 2, 'Service and endpoint boundaries must both be enforced by native');
  }
  assert.equal(businessReport.typedCommand.allowed, false);
  assert.equal(businessReport.typedCommand.status, 2, "Granting one IDL endpoint must not grant its siblings");
  assert.match(businessReport.read, /^denied:/, "Business API grants must not imply framework grants");
  assert.match(businessReport.write, /^denied:/);
  assert.equal(businessReport.createExplorer, false);
  await app.evaluate(() => window.capabilitiesDemo.businessHost.remove());
  await until(() => !pages().some((page) => page.url().includes("playground://business/")), "Business-only page leaked after host removal");

  // Forge an allowed application channel with a different wire command ID.
  const forged = await artifact.evaluate(async () => {
    const bytes = new Uint8Array(36);
    const data = new DataView(bytes.buffer);
    bytes.set([0x43, 0x45, 0x52, 0x4d]);
    data.setUint16(4, 1, true);
    data.setBigUint64(8, 1n, true);
    data.setBigUint64(16, 0x12345678n, true);
    try { await window.tuffite.invoke("playground.Commands.DoSomething", bytes); return "allowed"; }
    catch (error) { return { status: error.status, message: error.message }; }
  });
  assert.equal(forged.status, 2, "A forged channel must be rejected at the native authorization boundary");

  // Navigation keeps the native target but recomputes document-origin grants.
  const changed = new URL(preview.url());
  changed.hostname = "demo.artifact";
  changed.search = "?navigation-origin-check";
  await app.evaluate((url) => window.capabilitiesDemo.preview.load(url), changed.href);
  await preview.waitForURL(changed.href);
  await preview.waitForFunction(() => document.body.dataset.capabilityReport);
  assert.equal((await report(preview)).appCommand, "allowed");
  changed.hostname = "other.preview";
  await app.evaluate((url) => window.capabilitiesDemo.preview.load(url), changed.href);
  await preview.waitForURL(changed.href);
  await preview.waitForFunction(() => document.body.dataset.capabilityReport);
  assert.match((await report(preview)).appCommand, /^denied:/);
  const scripted = await app.evaluate(() => window.capabilitiesDemo.preview.executeJavaScript("location.origin"));
  assert.equal(scripted, "playground://other.preview");
  await app.evaluate(() => { window.capabilitiesDemo.artifact.remove(); window.capabilitiesDemo.preview.remove(); });
  await until(() => !pages().some((page) => /\.(artifact|preview)\//.test(page.url())), "Removed hosts retained live pages");
  console.log(JSON.stringify({ originCapabilities: "passed", typedUtilityAllowed: "passed", typedUtilityEndpointDenied: "passed", typedUtilityServiceDenied: "passed", fileScopes, assetsWithoutGrants: "passed", businessDoesNotGrantFramework: "passed", unmappedAssetStatus: 404, artifact: a, preview: p, wireChannelForgery: "denied", navigationRevocation: "passed", cleanup: "passed" }, null, 2));
} finally { await browser.close(); }

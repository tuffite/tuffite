import assert from "node:assert/strict";
import { createRequire } from "node:module";
import http from "node:http";
const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT);
const fixturePort = Number(process.env.TUFFITE_EXPLORER_RELOAD_HTTP_PORT);
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
const app = browser
  .contexts()
  .flatMap((context) => context.pages())
  .find((page) => page.url().includes("/behavior.html"));
assert.ok(app);
const server = http.createServer((_request, response) => {
  response.writeHead(200, {
    "content-type": "text/html",
    "content-security-policy": "script-src 'none'",
  });
  response.end(
    '<!doctype html><title>HTTP CSP fixture</title><script>window.serverPolicyEscaped=true</script><img id="blocked-image" src="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7">',
  );
});
await new Promise((resolve) =>
  server.listen(fixturePort, "127.0.0.1", resolve),
);
const source = app.locator("#behavior-source");
const state = () => source.evaluate((element) => element.getNavigationState());
const wait = async (probe, label) => {
  for (let i = 0; i < 200; i++) {
    if (await probe()) return;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`Timed out: ${label}`);
};
const targets = async () =>
  (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
// Isolated Explorer targets are not Playwright Pages. Use the debugger only
// for controlled fixture interaction; application code uses the public API.
const execute = async (expression) => {
  const url = (await state()).url;
  const target = (await targets()).find((entry) => entry.url === url);
  assert.ok(target, `Missing fixture target: ${url}`);
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  try {
    return await new Promise((resolve, reject) => {
      const timer = setTimeout(
        () => reject(new Error("Fixture debugger timed out")),
        8000,
      );
      socket.addEventListener("message", (event) => {
        const reply = JSON.parse(event.data);
        if (reply.id !== 1) return;
        clearTimeout(timer);
        if (reply.error || reply.result?.exceptionDetails)
          reject(
            new Error(
              JSON.stringify(reply.error || reply.result.exceptionDetails),
            ),
          );
        else resolve(reply.result.result.value);
      });
      socket.send(
        JSON.stringify({
          id: 1,
          method: "Runtime.evaluate",
          params: {
            expression,
            returnByValue: true,
            awaitPromise: true,
            userGesture: true,
          },
        }),
      );
    });
  } finally {
    socket.close();
  }
};
const click = (id) =>
  execute(`document.querySelector(${JSON.stringify(id)}).click(); true`);
try {
  await wait(
    async () =>
      (await state()).url.includes("fixtures/behavior.html") &&
      !(await state()).isLoading,
    "origin-scoped fixture",
  );
  await click("#navigate");
  await wait(
    () =>
      app.evaluate(() =>
        policyLab.events.some((event) => event.type === "navigation"),
      ),
    "delegated navigation",
  );
  assert.equal(new URL((await state()).url).search, "");
  const confirmation = click("#confirm");
  await app.locator("#confirmation").waitFor({ state: "visible" });
  await app.locator("#dismiss").click();
  await confirmation;
  assert.equal(await execute("document.body.dataset.confirmed"), "false");
  await click("#external");
  await app.locator("#confirmation").waitFor({ state: "visible" });
  assert.equal(
    await app.evaluate(() => policyLab.pending.initiatingOrigin),
    "playground://behavior.fixture",
  );
  await source.evaluate(async (element) => {
    element.policy += " context-menu: browser; freeze: never; discard: never;";
    await element.getNavigationState();
  });
  assert.equal(await app.evaluate(() => !!policyLab.pending && !policyLab.pending.signal.aborted),
    true, "Unrelated policy changes must preserve a pending decision");
  await app.locator("#dismiss").click();
  await wait(
    () => app.evaluate(() => policyLab.events.at(-1).type === "dismissed"),
    "external protocol dismissal",
  );
  assert.equal(
    await app.evaluate(() => policyLab.events.at(-1).result),
    true,
  );
  await source.evaluate((element) => {
    element.policy = "external-protocol: delegate timeout(40ms);";
  });
  await click("#external");
  await wait(
    () =>
      app.evaluate(() =>
        policyLab.events.some((event) => event.type === "canceled"),
      ),
    "request timeout",
  );
  await source.evaluate((element) => {
    element.policy = "external-protocol: block; navigation: browser;";
  });
  const count = await app.evaluate(
    () =>
      policyLab.events.filter((event) => event.type === "externalprotocol")
        .length,
  );
  await click("#external");
  await new Promise((resolve) => setTimeout(resolve, 100));
  assert.equal(
    await app.evaluate(
      () =>
        policyLab.events.filter((event) => event.type === "externalprotocol")
          .length,
    ),
    count,
  );
  await click("#navigate");
  await wait(
    async () => (await state()).url.endsWith("?navigated"),
    "browser navigation",
  );
  await source.evaluate((element) => {
    element.policy = "navigation: block;";
  });
  await execute(
    'document.querySelector("#navigate").href = "fixtures/behavior.html?blocked"; true',
  );
  await click("#navigate");
  await new Promise((resolve) => setTimeout(resolve, 100));
  assert.equal(new URL((await state()).url).search, "?navigated");
  await source.evaluate((element) => {
    element.policy = "navigation: browser; dialog: unknown;";
  });
  await click("#navigate");
  await new Promise((resolve) => setTimeout(resolve, 100));
  assert.equal(new URL((await state()).url).search, "?navigated");
  await source.evaluate((element) => {
    element.policy = "navigation: browser; new-window: browser;";
  });
  await click("#popup");
  await wait(
    async () => (await targets()).some((page) => page.url.endsWith("?popup")),
    "native browser popup",
  );
  const popup = (await targets()).find((page) => page.url.endsWith("?popup"));
  const browserSession = await browser.newBrowserCDPSession();
  await browserSession.send("Target.closeTarget", { targetId: popup.id });
  await source.evaluate(
    (element, url) => element.load(url),
    `http://127.0.0.1:${fixturePort}/`,
  );
  await wait(
    async () =>
      (await state()).url.startsWith("http:") && !(await state()).isLoading,
    "HTTP CSP document",
  );
  const csp = await execute(`({
    serverBlocked: !window.serverPolicyEscaped,
    manifestBlocked:
      document.querySelector("#blocked-image").complete &&
      document.querySelector("#blocked-image").naturalWidth === 0,
  })`);
  assert.deepEqual(csp, { serverBlocked: true, manifestBlocked: true });
  await source.evaluate((element) => {
    element.policy = "navigation: delegate; external-protocol: delegate;";
    element.load("playground-resource://main/ui/dist/fixtures/behavior.html");
  });
  await wait(
    async () =>
      (await state()).url.startsWith("playground-resource:") &&
      !(await state()).isLoading,
    "secondary application scheme",
  );
  assert.equal(await execute("location.origin"), "playground-resource://main");
  assert.equal(
    await execute("startSchemeWorker()"),
    "playground-resource://main",
  );
  assert.equal(await execute("fetch(location.href).then(r => r.ok)"), true);
  await execute('localStorage.setItem("scheme-isolation", "secondary"); true');
  assert.equal(
    await app.evaluate(() => localStorage.getItem("scheme-isolation")),
    null,
  );
  const navigationCount = await app.evaluate(
    () => policyLab.events.filter((e) => e.type === "navigation").length,
  );
  await click("#navigate");
  await wait(
    () =>
      app.evaluate(
        (count) =>
          policyLab.events.filter((e) => e.type === "navigation").length >
          count,
        navigationCount,
      ),
    "secondary scheme navigation delegation",
  );
  assert.equal(new URL((await state()).url).search, "");
  console.log(
    "browser_policy=passed origin defaults, delegate, browser, block, timeout, initiating origin, HTTP/server CSP intersection, secondary scheme assets/fetch/Worker/storage/navigation",
  );
} finally {
  await browser.close();
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
}

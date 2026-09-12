import assert from "node:assert/strict";
import { createRequire } from "node:module";

const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");

const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT ?? "9257");
let browser;
let app;

try {
  phase("navigate-explorer-only-target");
  await navigateMainTarget();
  phase("raw-dialog-controls");
  await verifyDialogBridgeWithoutDevToolsDialogHandling();
  phase("connect-cdp");
  browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, {
    timeout: 15_000,
  });
  // Explorer documents are separate CDP page targets. Install the no-op
  // handler on every current and future target so Playwright does not race
  // the Tuffite dialog bridge by auto-dismissing a dialog that the demo is
  // about to accept or dismiss through `detail`.
  for (const context of browser.contexts()) {
    const preserveTuffiteDialog = (page) => page.on("dialog", () => {});
    context.pages().forEach(preserveTuffiteDialog);
    context.on("page", preserveTuffiteDialog);
  }
  app = await waitFor(() =>
    browser
      .contexts()
      .flatMap((context) => context.pages())
      .find((page) => page.url().includes("/index.html?explorer-smoke")),
  );
  app.setDefaultTimeout(15_000);
  const diagnostics = [];
  app.on("crash", () => diagnostics.push("page:crash"));
  app.on("pageerror", (error) =>
    diagnostics.push(`page:error:${error.message}`),
  );
  phase("wait-self-checks");
  await app
    .locator("#explorer-suite-status.ready")
    .waitFor({ timeout: 45_000 });
  await app.getByText("Core checks passed", { exact: true }).waitFor();
  assert.equal(await app.locator("#thread-explorer-slot explorer").count(), 1);
  assert.equal(
    await app.locator("#isolated-explorer-slot explorer").count(),
    0,
  );
  const bridgeContract = await app.evaluate(() => ({
    hasRootListener: typeof window.tuffite?.addEventListener === "function",
    hasChannelNamespace: "channel" in (window.tuffite ?? {}),
    leaksChannelSink: "resetChannelSink" in (window.tuffite ?? {}),
    leaksMessageSink: "resetMessageSink" in (window.tuffite ?? {}),
  }));
  assert.deepEqual(bridgeContract, {
    hasRootListener: true,
    hasChannelNamespace: false,
    leaksChannelSink: false,
    leaksMessageSink: false,
  });

  phase("parent-api");
  const processState = await app
    .locator("#thread-explorer-slot explorer")
    .evaluate((element) => element.getNavigationState());
  assert.equal(processState.rendererProcess?.isolated, true);
  assert.notEqual(
    processState.rendererProcess?.id,
    processState.rendererProcess?.ownerId,
  );
  await app.locator("#explorer-state").click();
  await expectText(app, "#explorer-command-result", "storagePartition");
  await app.locator("#explorer-execute").click();
  await expectText(app, "#explorer-command-result", "executeJavaScript");
  await app.locator("#explorer-message").click();
  await expectText(app, "#explorer-action-status", "postMessage completed");
  await expectText(app, "#explorer-events", "parent-message-ack");

  phase("navigation-api");
  await app.locator("#explorer-history-a").click();
  await waitForState(app, "history-a.html");
  await app.locator("#explorer-history-b").click();
  await waitForState(app, "history-b.html");
  await app.locator("#explorer-back").click();
  await waitForState(app, "history-a.html");
  await app.locator("#explorer-forward").click();
  await waitForState(app, "history-b.html");
  await app.locator("#explorer-go-back").click();
  await waitForState(app, "history-a.html");
  await app.locator("#explorer-forward").click();
  await waitForState(app, "history-b.html");
  await clickAndComplete(app, "#explorer-reload", "reload completed");
  await clickAndComplete(
    app,
    "#explorer-reload-cache",
    "reload(bypassCache) completed",
  );
  await clickAndComplete(app, "#explorer-stop", "stop completed");
  await app.locator("#explorer-home").click();
  await waitForState(app, "explorer.html");

  phase("open-kinds");
  await app
    .locator("#thread-explorer-slot explorer")
    .evaluate(async (element) => {
      const state = await element.getNavigationState();
      await element.executeJavaScript(
        `["#navigation-link", "#window-open-button", "#popup-link"]` +
          `.forEach((selector) => ` +
          `document.querySelector(selector).click()); true`,
        { expectedDocumentId: state.documentId },
      );
    });
  await expectText(app, "#explorer-events", "navigation");
  await expectText(app, "#explorer-events", "open");
  await expectText(app, "#explorer-events", "popup");
  const newWindowPoint = await app
    .locator("#thread-explorer-slot explorer")
    .evaluate(async (element) => {
      const state = await element.getNavigationState();
      return element.executeJavaScript(
        `(() => { const rect = document.querySelector(` +
          `'#new-window-link').getBoundingClientRect(); ` +
          `return {x: rect.left + rect.width / 2, ` +
          `y: rect.top + rect.height / 2}; })()`,
        { expectedDocumentId: state.documentId },
      );
    });
  const explorerBounds = await app
    .locator("#thread-explorer-slot explorer")
    .boundingBox();
  assert.ok(explorerBounds, "Explorer has no clickable bounds");
  await app.keyboard.down("Shift");
  await app.mouse.click(
    explorerBounds.x + newWindowPoint.x,
    explorerBounds.y + newWindowPoint.y,
  );
  await app.keyboard.up("Shift");
  await expectText(app, "#explorer-events", "newwindow");

  phase("nested-request-bubbling");
  await app.locator("#explorer-clear-events").click();
  await app.locator("#explorer-nested-open-bubble").click();
  await expectText(app, "#explorer-events", "level-1-element:open");
  await expectText(app, "#explorer-events", "top:open");

  await app.locator("#explorer-clear-events").click();
  await app.locator("#explorer-nested-open-stop").click();
  await expectText(app, "#explorer-events", '"stopped": true');
  await app.waitForTimeout(750);
  assert.equal(
    await app
      .locator("#explorer-events > li", { hasText: "top:open" })
      .count(),
    0,
  );

  await app.locator("#explorer-dialog-mode").selectOption("auto");
  await app.locator("#explorer-clear-events").click();
  await app.locator("#explorer-nested-dialog-bubble").click();
  await expectText(app, "#explorer-events", "level-1-element:dialog");
  await expectText(app, "#explorer-events", "top:dialog");
  await expectText(app, "#explorer-events", "nested:dialog-result");

  await app.locator("#explorer-clear-events").click();
  await app.locator("#explorer-nested-dialog-stop").click();
  await expectText(app, "#explorer-events", "level-1-element:dialog");
  await expectText(app, "#explorer-events", '"stopped": true');
  await expectText(app, "#explorer-events", "nested:dialog-result");
  await app.waitForTimeout(750);
  assert.equal(
    await app
      .locator("#explorer-events > li", { hasText: "top:dialog" })
      .count(),
    0,
  );

  phase("lifecycle");
  await app.locator("#explorer-recreate").click();
  await waitForState(app, "explorer.html");
  await app.locator("#explorer-stress").click();
  await expectText(
    app,
    "#explorer-command-result",
    "Stress ×20 completed",
    90_000,
  );

  phase("storage-partition-matrix");
  await app.locator("#explorer-storage").click();
  await expectText(
    app,
    "#explorer-storage-result",
    '"name": "playground-v2-persist"',
    45_000,
  );
  await expectText(
    app,
    "#explorer-storage-result",
    '"name": "playground-v2-memory"',
  );
  await expectText(
    app,
    "#explorer-storage-result",
    '"name": "playground-v2-inherit-persist"',
  );
  await expectText(
    app,
    "#explorer-storage-result",
    '"name": "playground-v2-inherit-memory"',
  );
  await expectText(app, "#explorer-storage-result", '"mode": "shared"');
  await expectText(app, "#explorer-storage-result", '"mode": "persist"');
  await expectText(app, "#explorer-storage-result", '"mode": "memory"');
  await expectText(
    app,
    "#explorer-storage-result",
    '"mode": "inherit+persist"',
  );
  await expectText(app, "#explorer-storage-result", '"mode": "inherit+memory"');
  await expectText(app, "#explorer-storage-result", '"inherited": true');
  await expectText(app, "#explorer-storage-result", '"inMemory": true');

  phase("context-menu-toggle");
  await app.locator("#explorer-context-menu").uncheck();
  await expectText(app, "#explorer-action-status", "context menu disabled");
  await app.locator("#explorer-context-menu").check();
  await expectText(app, "#explorer-action-status", "context menu enabled");

  phase("load-url-control");
  await app
    .locator("#explorer-url")
    .fill("playground://main/ui/dist/history-a.html?from=url-control");
  await app.locator("#explorer-load-url").click();
  await waitForState(app, "history-a.html?from=url-control");

  phase("baidu-native-context-menu");
  await app.locator("#explorer-baidu").click();
  await waitForState(app, "https://www.baidu.com/", 60_000);
  // CDP dispatches renderer keyboard events, not AppKit events. A real macOS
  // menu cannot be dismissed with that Escape and requires a native input
  // driver. Its enable/disable policy is checked above on every platform.
  const nativeMenuInteraction = process.platform !== "darwin";
  if (nativeMenuInteraction) {
    const explorerBox = await app
      .locator("#thread-explorer-slot explorer")
      .boundingBox();
    assert.ok(explorerBox, "Explorer has no clickable bounds");
    for (let attempt = 0; attempt < 3; attempt += 1) {
      await app.mouse.click(
        explorerBox.x + explorerBox.width / 2,
        explorerBox.y + explorerBox.height / 2,
        { button: "right" },
      );
      await app.waitForTimeout(400);
      await app.keyboard.press("Escape");
    }
  }
  await app.locator("#explorer-state").click();
  await expectText(app, "#explorer-command-result", "www.baidu.com");

  phase("clear-events-control");
  await app.locator("#explorer-clear-events").click();
  await expectText(app, "#explorer-action-status", "Event log cleared");
  assert.equal(await app.locator("#explorer-events > li").count(), 0);
  assert.equal(await app.locator("#explorer-suite-status.ready").count(), 1);

  assert.deepEqual(diagnostics, []);
  process.stdout.write(
    `${JSON.stringify({
      status: "ok",
      selfChecks: true,
      channelBridgeContract: true,
      parentApi: true,
      rendererIsolation: true,
      navigationApi: true,
      dialogResponses: true,
      partitionIsLazy: true,
      isolatedPartition: true,
      inheritedPartitions: true,
      openRequests: true,
      nestedRequestBubbling: true,
      nestedStopPropagation: true,
      lifecycleStress: true,
      remoteNavigation: true,
      nativeContextMenu: true,
      nativeContextMenuInteraction: nativeMenuInteraction,
      diagnostics,
    })}\n`,
  );
  process.exit(0);
} catch (error) {
  let state;
  try {
    state = await app?.evaluate(() => ({
      title: document.title,
      suite: document.querySelector("#explorer-suite-status")?.textContent,
      checks: document.querySelector("#explorer-v2")?.textContent,
      action: document.querySelector("#explorer-action-status")?.textContent,
      command: document.querySelector("#explorer-command-result")?.textContent,
      events: [...document.querySelectorAll("#explorer-events > li")]
        .slice(0, 12)
        .map((item) => item.textContent),
    }));
  } catch (diagnosticError) {
    state = {
      diagnosticError: diagnosticError?.message ?? String(diagnosticError),
    };
  }
  process.stderr.write(
    `[explorer-demo-smoke] failed: ${error?.stack ?? error}\n` +
      `[explorer-demo-smoke] state: ${JSON.stringify(state)}\n`,
  );
  process.exit(1);
}

async function clickAndComplete(page, selector, completion) {
  await page.locator(selector).click();
  await expectText(page, "#explorer-action-status", completion);
}

async function waitForState(page, includes, timeout = 30_000) {
  await waitFor(async () => {
    const state = await page
      .locator("#thread-explorer-slot explorer")
      .evaluate((element) => element.getNavigationState());
    return state.url.includes(includes);
  }, timeout);
}

async function verifyDialogBridgeWithoutDevToolsDialogHandling() {
  const targets = await waitFor(async () => {
    const response = await fetch(`http://127.0.0.1:${port}/json/list`);
    if (!response.ok) return undefined;
    return (await response.json()).find(
      (target) =>
        target.type === "page" &&
        target.url?.includes("/index.html?explorer-smoke") &&
        target.webSocketDebuggerUrl,
    );
  });
  const socket = await openSocket(targets.webSocketDebuggerUrl);
  let id = 100;
  const evaluate = async (expression) => {
    const result = await sendCdp(socket, ++id, "Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (result.result?.exceptionDetails) {
      throw new Error(JSON.stringify(result.result.exceptionDetails));
    }
    return result.result?.result?.value;
  };
  try {
    await sendCdp(socket, ++id, "Runtime.enable");
    await waitFor(
      () =>
        evaluate(
          "document.querySelector('#explorer-suite-status')?.classList.contains('ready')",
        ),
      45_000,
    );
    const processState = await evaluate(
      "document.querySelector('#thread-explorer-slot explorer').getNavigationState()",
    );
    assert.equal(processState.rendererProcess?.isolated, true);
    assert.notEqual(
      processState.rendererProcess?.id,
      processState.rendererProcess?.ownerId,
    );
    await evaluate(
      "document.querySelector('#explorer-dialog-mode').value = 'manual'; true",
    );
    for (const [kind, accept] of [
      ["alert", true],
      ["confirm", false],
      ["prompt", true],
      ["beforeunload", true],
    ]) {
      phase(`raw-dialog-${kind}`);
      if (kind === "beforeunload") {
        await sendCdp(socket, ++id, "Page.bringToFront");
        const point = await evaluate(`(async () => {
          const element = document.querySelector('#thread-explorer-slot explorer');
          element.scrollIntoView({block: 'center', behavior: 'instant'});
          await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
          const rect = element.getBoundingClientRect();
          return {x: rect.left + rect.width / 2, y: rect.top + 40};
        })()`);
        await sendCdp(socket, ++id, "Input.dispatchMouseEvent", {
          type: "mousePressed",
          x: point.x,
          y: point.y,
          button: "left",
          buttons: 1,
          clickCount: 1,
        });
        await sendCdp(socket, ++id, "Input.dispatchMouseEvent", {
          type: "mouseReleased",
          x: point.x,
          y: point.y,
          button: "left",
          buttons: 0,
          clickCount: 1,
        });
        if (!await evaluate(
          "document.querySelector('#thread-explorer-slot explorer').executeJavaScript('navigator.userActivation.hasBeenActive')",
        )) {
          const candidates = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
          const child = candidates.find(candidate =>
            candidate.type === "page" && candidate.url?.includes("/explorer.html?mode=complete"));
          assert.ok(child?.webSocketDebuggerUrl, "Explorer child CDP target is missing");
          const childSocket = await openSocket(child.webSocketDebuggerUrl);
          try {
            for (const type of ["mousePressed", "mouseReleased"]) {
              await sendCdp(childSocket, ++id, "Input.dispatchMouseEvent", {
                type, x: 60, y: 40, button: "left",
                buttons: type === "mousePressed" ? 1 : 0, clickCount: 1,
              });
            }
          } finally {
            childSocket.close();
          }
        }
        await waitFor(() => evaluate(
          "document.querySelector('#thread-explorer-slot explorer').executeJavaScript('navigator.userActivation.hasBeenActive')",
        ));
      }
      await evaluate(
        `setTimeout(() => document.querySelector('#explorer-${kind}').click(), 0); true`,
      );
      await waitFor(() =>
        evaluate(
          `document.querySelector('#explorer-dialog-result')?.textContent.includes('\"type\": \"${kind}\"')`,
        ),
      );
      await evaluate(
        `document.querySelector('${
          accept ? "#explorer-dialog-accept" : "#explorer-dialog-reject"
        }').click(); true`,
      );
      await waitFor(() =>
        evaluate(
          `document.querySelector('#explorer-dialog-result')?.textContent.includes('${
            accept ? "Accepted" : "Cancelled"
          } ${kind}: true')`,
        ),
      );
      if (kind === "beforeunload") {
        // navigation=emit converts the scripted reload into an open event, so
        // the source document remains active and retains its handler. Remove
        // it before the later parent-driven navigation API checks.
        await evaluate(
          `document.querySelector('#thread-explorer-slot explorer')` +
            `.executeJavaScript('window.onbeforeunload = null; true')`,
        );
      }
    }
  } finally {
    socket.close();
  }
}

async function expectText(page, selector, expected, timeout = 30_000) {
  await waitFor(
    async () =>
      (await page.locator(selector).textContent())?.includes(expected),
    timeout,
  );
}

function phase(name) {
  process.stderr.write(`[explorer-demo-smoke] ${name}\n`);
}

async function navigateMainTarget() {
  const targets = await waitFor(async () => {
    const response = await fetch(`http://127.0.0.1:${port}/json/list`);
    if (!response.ok) return undefined;
    const candidates = await response.json();
    return candidates.find(
      (candidate) =>
        candidate.type === "page" &&
        candidate.url?.includes("/index.html") &&
        candidate.webSocketDebuggerUrl,
    );
  });
  const url = `${targets.url.split(/[?#]/, 1)[0]}?explorer-smoke`;
  const socket = new WebSocket(targets.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error("Timed out opening main-page CDP target")),
      10_000,
    );
    socket.addEventListener(
      "open",
      () => {
        clearTimeout(timeout);
        resolve();
      },
      { once: true },
    );
    socket.addEventListener("error", reject, { once: true });
  });
  try {
    await sendCdp(socket, 1, "Page.enable");
    await sendCdp(socket, 2, "Page.navigate", { url });
    await waitFor(async () => {
      const result = await sendCdp(socket, 3, "Runtime.evaluate", {
        expression:
          "location.search.includes('explorer-smoke') && document.readyState !== 'loading'",
        returnByValue: true,
      });
      return result.result?.result?.value === true;
    });
  } finally {
    socket.close();
  }
}

function openSocket(url) {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(url);
    const timeout = setTimeout(
      () => reject(new Error(`Timed out opening ${url}`)),
      10_000,
    );
    socket.addEventListener(
      "open",
      () => {
        clearTimeout(timeout);
        resolve(socket);
      },
      { once: true },
    );
    socket.addEventListener("error", reject, { once: true });
  });
}

function sendCdp(socket, id, method, params = {}) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error(`Timed out calling ${method}`)),
      10_000,
    );
    const onMessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.id !== id) return;
      clearTimeout(timeout);
      socket.removeEventListener("message", onMessage);
      if (message.error) reject(new Error(JSON.stringify(message.error)));
      else resolve(message);
    };
    socket.addEventListener("message", onMessage);
    socket.send(JSON.stringify({ id, method, params }));
  });
}

async function waitFor(probe, timeout = 30_000) {
  const deadline = Date.now() + timeout;
  let value;
  while (Date.now() < deadline) {
    value = await probe();
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`Timed out; last=${JSON.stringify(value)}`);
}

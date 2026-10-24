import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");
const browser = await chromium.connectOverCDP(
  `http://127.0.0.1:${process.env.TUFFITE_PLAYGROUND_CDP_PORT}`,
);
const cdp = await browser.newBrowserCDPSession();
const until = async (probe, description) => {
  const end = Date.now() + 20000;
  while (Date.now() < end) {
    const value = await probe();
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`Timed out: ${description}`);
};
const denied = /not allowed|denied|not permitted|No target|not found/i;
try {
  if (process.env.TUFFITE_DEVTOOLS_ROOT_DENIED === "1") {
    const target = await until(
      async () =>
        (await cdp.send("Target.getTargets")).targetInfos.find(
          (target) => target.url === process.env.TUFFITE_DEVTOOLS_ROOT_URL,
        ),
      "root target",
    );
    await assert.rejects(
      () =>
        cdp.send("Target.attachToTarget", {
          targetId: target.targetId,
          flatten: true,
        }),
      denied,
    );
    console.log("devtools_origin_root=ok attach_denied=true");
  } else {
    const app = await until(
      () =>
        browser
          .contexts()
          .flatMap((context) => context.pages())
          .find((page) =>
            /\/(?:index.html|main)$/.test(new URL(page.url()).pathname),
          ),
      "application page",
    );
    assert.equal(
      await app.evaluate(
        () => "devtools" in document.createElement("explorer"),
      ),
      false,
      "DevTools policy must not be exposed as an Explorer property",
    );
    const allowedUrl = process.env.TUFFITE_DEVTOOLS_ALLOWED_URL;
    const deniedUrl = process.env.TUFFITE_DEVTOOLS_DENIED_URL;
    await app.evaluate(async (url) => {
      const explorer = document.createElement("explorer");
      explorer.id = "origin-policy-fixture";
      explorer.navigation = "allow";
      explorer.style.cssText = "display:block;width:400px;height:300px";
      explorer.src = url;
      const ready = new Promise((resolve) =>
        explorer.addEventListener("loadstop", resolve, { once: true }),
      );
      document.body.append(explorer);
      await ready;
    }, allowedUrl);
    const target = await until(
      async () =>
        (await cdp.send("Target.getTargets")).targetInfos.find(
          (target) => target.url === allowedUrl,
        ),
      "allowed origin target",
    );
    const session = await cdp.send("Target.attachToTarget", {
      targetId: target.targetId,
      flatten: true,
    });
    let detached = false;
    cdp.on("Target.detachedFromTarget", (event) => {
      if (event.sessionId === session.sessionId) detached = true;
    });
    await app.evaluate(async (url) => {
      const explorer = document.querySelector("#origin-policy-fixture");
      const ready = new Promise((resolve) =>
        explorer.addEventListener("loadstop", resolve, { once: true }),
      );
      explorer.load(url);
      await ready;
    }, deniedUrl);
    await until(() => detached, "inspector detached after origin change");
    await assert.rejects(
      () =>
        cdp.send("Target.attachToTarget", {
          targetId: target.targetId,
          flatten: true,
        }),
      denied,
    );
    console.log(
      "devtools_origin_explorer=ok allowed_origin=true denied_origin=true navigation_detaches=true host_controlled=true",
    );
  }
} finally {
  await cdp.detach();
  await browser.close();
}

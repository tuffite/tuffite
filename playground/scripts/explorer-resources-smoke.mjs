import assert from "node:assert/strict";
import http from "node:http";
import { createRequire } from "node:module";
const require = createRequire(new URL("../ui/package.json", import.meta.url));
const { chromium } = require("playwright-core");
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT ?? "9257");
const pdf = Buffer.from("%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Count 0/Kids[]>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF");
let cookieObserved = false;
const server = http.createServer((request, response) => {
  if (request.url === "/document.pdf") {
    cookieObserved = request.headers.cookie?.includes("resource-session=present") ?? false;
    response.writeHead(200, { "content-type": "application/pdf", "content-length": pdf.length });
    response.end(pdf);
  } else if (request.url === "/slow.bin") {
    response.writeHead(200, { "content-type": "application/octet-stream", "content-length": 1024 * 1024 });
    const timer = setInterval(() => response.write(Buffer.alloc(1024)), 20);
    response.on("close", () => clearInterval(timer));
  } else {
    response.writeHead(200, { "content-type": "text/html", "set-cookie": "resource-session=present; Path=/; SameSite=Lax" });
    response.end("<!doctype html><title>Explorer resources</title><body style='background:#184a70'><h1>Resources</h1><canvas id='canvas' width='320' height='180'></canvas><video id='video' muted autoplay playsinline></video><script>canvas.getContext('2d').fillRect(0,0,320,180);video.srcObject=canvas.captureStream(10);video.play();</script>");
  }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const url = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
// Playwright redirects downloads to GUID-only temporary filenames by default.
// Exercise the Framework's real delegate and target naming instead.
const browserSession = await browser.newBrowserCDPSession();
await browserSession.send("Browser.setDownloadBehavior", { behavior: "default" });
const pages = () => browser.contexts().flatMap((context) => context.pages());
async function until(check, description) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const result = await check();
    if (result) return result;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(description);
}
try {
  const app = await until(() => pages().find((page) => page.url().includes("/index.html")), "Missing Playground");
  await app.evaluate(async (url) => {
    const host = document.createElement("div");
    host.style.cssText = "position:fixed;inset:0;background:#184a70;z-index:10000";
    document.body.append(host);
    const source = document.createElement("explorer");
    source.navigation = "allow";
    source.dialogs = "emit";
    source.contextMenuEnabled = true;
    source.style.cssText = "display:block;width:100%;height:100%";
    window.resourceEvents = [];
    source.addEventListener("download", ({ detail }) => resourceEvents.push(detail));
    const ready = new Promise((resolve) => source.addEventListener("loadstop", resolve, { once: true }));
    source.src = url;
    host.append(source);
    window.resourceSource = source;
    await ready;
  }, url);
  const preview = await until(async () => {
    const data = await app.evaluate(() => resourceSource.capturePreview());
    return data || false;
  }, "Capture never received a painted native surface");
  assert.ok(preview.startsWith("data:image/png;base64,"), "Capture must return a painted native surface");
  await app.evaluate((url) => resourceSource.load(`${url}/document.pdf`), url);
  const item = await until(() => app.evaluate(() => resourceEvents.find((item) => item.state === "complete" && item.mimeType === "application/pdf")), "PDF did not download");
  assert.equal(item.navigation, true);
  assert.equal(cookieObserved, true, "PDF navigation must retain Explorer cookies");
  assert.ok(item.filename.endsWith("document.pdf"), JSON.stringify(item));
  assert.ok(item.path, "Completed download must have a target path");
  const data = await app.evaluate((id) => resourceSource.readDownload(id), item.id);
  assert.deepEqual(Buffer.from(data, "base64"), pdf);
  assert.equal(await app.evaluate(() => resourceSource.download("javascript:alert(1)")), false);
  assert.ok(item.path.includes("/Downloads/"), "Framework delegate must choose the download destination");
  console.log("PDF download, cookies and bytes passed");
  // An unrelated receiver must not be able to read or cancel another page's item.
  const unowned = await app.evaluate(async ({ url, id }) => {
    const other = document.createElement("explorer");
    other.navigation = "allow";
    other.src = url;
    document.body.append(other);
    await new Promise((resolve) => other.addEventListener("loadstop", resolve, { once: true }));
    let rejected = false;
    try { await other.readDownload(id); } catch { rejected = true; }
    const cancelled = await other.cancelDownload(id);
    other.remove();
    return { rejected, cancelled };
  }, { url, id: item.id });
  assert.deepEqual(unowned, { rejected: true, cancelled: false });
  const transferred = await app.evaluate(async (id) => {
    const source = resourceSource;
    const target = document.createElement("explorer");
    target.deferred = true;
    target.navigation = "allow";
    target.dialogs = "emit";
    target.style.cssText = source.style.cssText;
    source.parentElement.append(target);
    const transfer = await source.detach();
    await target.attach(transfer.token);
    const items = await target.getDownloads();
    const data = await target.readDownload(id);
    source.remove();
    window.resourceSource = target;
    target.addEventListener("download", ({ detail }) => resourceEvents.push(detail));
    return { items, data };
  }, item.id);
  assert.ok(transferred.items.find((value) => value.id === item.id && value.navigation));
  assert.equal(transferred.data, data);
  console.log("Download ownership and transfer replay passed");
  assert.equal(await app.evaluate((url) => resourceSource.download(`${url}/slow.bin`), url), true);
  const slow = await until(() => app.evaluate(() => resourceEvents.find((item) => item.url.endsWith("slow.bin") && item.receivedBytes > 0)), "Missing download progress");
  assert.equal(await app.evaluate((id) => resourceSource.cancelDownload(id), slow.id), true);
  await until(() => app.evaluate((id) => resourceEvents.some((item) => item.id === id && item.state === "cancelled"), slow.id), "Cancel did not emit a terminal state");
  console.log("Download progress and cancellation passed");
  await app.evaluate((url) => resourceSource.load(url), url);
  const guest = await until(() => pages().find((page) => page.url() === `${url}/`), "Video fixture missing");
  await guest.waitForFunction(() => video.readyState >= 2);
  await guest.evaluate(async () => { await video.requestPictureInPicture(); });
  assert.equal(await guest.evaluate(() => document.pictureInPictureElement === video), true);
  await guest.evaluate(() => document.exitPictureInPicture());
  console.log("Video PiP enter and exit passed");
  for (let attempt = 0; attempt < 2; attempt++) {
    const result = await guest.evaluate(async () => {
      const pip = await Promise.race([
        documentPictureInPicture.requestWindow({ width: 420, height: 240 }),
        new Promise((_, reject) => setTimeout(() => reject(new Error("Document PiP timed out")), 15000)),
      ]);
      pip.document.body.textContent = "Document PiP works";
      const text = pip.document.body.textContent;
      const related = pip.opener === window;
      pip.close();
      return { text, related };
    });
    assert.equal(result.text, "Document PiP works");
    assert.equal(result.related, true);
    console.log(`Document PiP open/close ${attempt + 1} passed`);
  }
  await app.evaluate(() => resourceSource.remove());
  await app.evaluate((url) => {
    const direct = document.createElement("explorer");
    direct.navigation = "allow";
    direct.style.cssText = "display:block;width:600px;height:400px";
    direct.src = `${url}/document.pdf`;
    document.body.append(direct);
    window.resourceDirect = direct;
  }, url);
  await until(() => app.evaluate(async () => (await resourceDirect.getDownloads()).some((item) => item.state === "complete" && item.navigation)), "An initial PDF URL must attach and retain its download");
  assert.equal(await app.evaluate(async () => (await resourceDirect.getNavigationState()).url), `${url}/document.pdf`);
  await app.evaluate(() => resourceDirect.remove());
  console.log("Initial PDF URL attachment passed");
  console.log("Explorer resources smoke passed: PDF cookies/bytes, ownership, transfer, progress/cancel, video and document PiP");
} finally {
  await browser.close();
  await new Promise((resolve) => server.close(resolve));
}

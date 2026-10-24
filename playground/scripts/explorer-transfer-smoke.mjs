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
const appOrigin = await app.evaluate(() => location.origin);
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
  await app.waitForFunction(() => window.transferDemo);
  const initial = await app.evaluate(() => window.transferDemo.load());
  console.log("handle phase: exclusive contract");
  const contract = await app.evaluate(async () => {
    const demo = window.transferDemo;
    await demo.detach();
    const handle = demo.transfer;
    let clone, forged;
    try { structuredClone(handle); } catch (error) { clone = error.name; }
    try { await demo.second.attach("forged-token"); } catch (error) { forged = error.name; }
    const publicToken = "token" in handle;
    const adopting = demo.attach();
    let concurrent;
    try { await demo.first.attach(handle); } catch (error) { concurrent = error.name; }
    const after = await adopting;
    let duplicate;
    try { await demo.first.attach(handle); } catch (error) { duplicate = error.name; }
    await handle.close();
    return {publicToken, className: handle.constructor.name, clone, forged, concurrent, duplicate, state: handle.state, src: demo.active.getAttribute("src"), after};
  });
  assert.equal(contract.publicToken, false);
  assert.equal(contract.className, "ExplorerHandle");
  assert.equal(contract.clone, "DataCloneError");
  assert.equal(contract.forged, "TypeError");
  assert.equal(contract.concurrent, "InvalidStateError");
  assert.equal(contract.duplicate, "InvalidStateError");
  assert.equal(contract.state, "transferred");
  assert.equal(contract.src, null);
  assert.equal(contract.after.state.documentId, initial.state.documentId);
  assert.equal(contract.after.marker, initial.marker);
  const started = Date.now();
  await app.evaluate(async () => { for (let i=0;i<25;i++) { await window.transferDemo.detach(); await window.transferDemo.attach(); } });
  const stressMs = Date.now()-started;

  const replacement = await app.evaluate(async () => {
    const demo = window.transferDemo;
    const target = document.createElement("explorer");
    target.navigation="allow"; target.dialogs="emit";
    const loaded = new Promise(resolve => target.addEventListener("loadstop", resolve, {once:true}));
    target.src = new URL("history-a.html?replacement", location.href).href;
    document.querySelector("#hosts").append(target); await loaded;
    await target.executeJavaScript("globalThis.replacementMarker='old-page'");
    const original = await target.getNavigationState();
    const declared = target.getAttribute("src");
    const incompatible = document.createElement("explorer");
    incompatible.navigation="emit"; incompatible.dialogs="emit";
    const ready = new Promise(resolve => incompatible.addEventListener("loadstop", resolve, {once:true}));
    incompatible.src=new URL("history-b.html",location.href).href;
    document.querySelector("#hosts").append(incompatible); await ready;
    const wrongHandle=await incompatible.detach();
    let failure;
    try { await target.attach(wrongHandle); } catch (error) { failure=error.name; }
    const restored = await target.getNavigationState();
    const restoredMarker = await target.executeJavaScript("globalThis.replacementMarker");
    const preservedSrc=target.getAttribute("src")===declared;
    const pending=wrongHandle.state;
    await wrongHandle.close(); incompatible.remove();
    await demo.detach(); await target.attach(demo.transfer);
    const adopted=await target.getNavigationState();
    const cleared=target.getAttribute("src");
    target.src=new URL("history-b.html?after-attach",location.href).href;
    await new Promise(resolve => target.addEventListener("loadstop",resolve,{once:true}));
    const navigated=await target.getNavigationState();
    target.remove();
    return {failure, original, restored, restoredMarker, preservedSrc, pending, adopted, cleared, navigated};
  });
  assert.equal(replacement.failure,"SecurityError");
  assert.equal(replacement.original.documentId,replacement.restored.documentId);
  assert.equal(replacement.restoredMarker,"old-page");
  assert.equal(replacement.preservedSrc,true);
  assert.equal(replacement.pending,"pending");
  assert.equal(replacement.adopted.documentId,initial.state.documentId);
  assert.equal(replacement.cleared,null);
  assert.ok(replacement.navigated.url.includes("after-attach"));
  console.log("handle phase: replacement and rollback passed");

  await app.evaluate(async () => {
    await window.transferDemo.load();
    const abandoned = await window.transferDemo.active.detach();
    if (abandoned.state !== "pending") throw new Error("Expected a pending handle");
  });
  const gc = await app.context().newCDPSession(app);
  await gc.send("HeapProfiler.collectGarbage");
  await gc.detach();
  await until(async()=>await targetCount()===0,"Garbage collection retained an unreachable handle");
  console.log("handle phase: GC passed, early delivery");

  await app.evaluate(() => window.transferDemo.load());
  const nativeClone = await app.evaluate(async () => {
    await window.transferDemo.detach();
    const handle = window.transferDemo.transfer;
    let failed, duplicate, broadcast;
    try { structuredClone({handle,bad:()=>{}},{transfer:[handle]}); } catch(e) { failed=e.name; }
    const preserved = handle.state;
    try { structuredClone(handle,{transfer:[handle,handle]}); } catch(e) { duplicate=e.name; }
    const channel = new BroadcastChannel("no-handle-clones");
    try { channel.postMessage(handle); } catch(e) { broadcast=e.name; }
    channel.close();
    const received = structuredClone({handle},{transfer:[handle]}).handle;
    const state = handle.state;
    await window.transferDemo.active.attach(received);
    return {failed,preserved,duplicate,broadcast,state,receiverState:received.state,
      snapshot:await window.transferDemo.snapshot()};
  });
  assert.equal(nativeClone.failed,"DataCloneError");
  assert.equal(nativeClone.preserved,"pending");
  assert.equal(nativeClone.duplicate,"DataCloneError");
  assert.equal(nativeClone.broadcast,"DataCloneError");
  assert.equal(nativeClone.state,"transferred");
  assert.equal(nativeClone.receiverState,"transferred");
  const portTransfer = await app.evaluate(async () => {
    const handle = await window.transferDemo.active.detach();
    const channel = new MessageChannel();
    const result = new Promise((resolve,reject) => {
      channel.port2.onmessage = async event => {
        try {
          const received = event.data.handle;
          const pending = received.state;
          await window.transferDemo.active.attach(received);
          resolve({standard:event instanceof MessageEvent,pending,source:handle.state,
            same:received === handle,snapshot:await window.transferDemo.snapshot()});
        } catch(error) {reject(error);} finally {channel.port1.close();channel.port2.close();}
      };
    });
    channel.port1.postMessage({handle},{transfer:[handle]});
    return result;
  });
  assert.equal(portTransfer.standard,true);
  assert.equal(portTransfer.pending,"pending");
  assert.equal(portTransfer.source,"transferred");
  assert.equal(portTransfer.same,false);
  console.log("handle phase: structured clone and MessagePort transfer passed");
  const windowMessages = await app.evaluate(async () => {
    const win = await (await import('/tuffite.js')).window.current();
    const id = win.id;
    win.id = 0n;
    const buffer = new Uint8Array([1,2,3]).buffer;
    const cyclic = {big:42n, date:new Date(0), map:new Map([["key", "value"]]), buffer};
    cyclic.self = cyclic;
    const channel = new MessageChannel();
    const received = new Promise(resolve => {
      const listener = event => {
        if (event.data?.type !== "window-message-probe") return;
        window.removeEventListener("message",listener);
        event.ports[0].postMessage("ack");
        event.ports[0].close();
        resolve({standard:event instanceof MessageEvent, trusted:event.isTrusted,
          source:event.source, origin:event.origin, ports:event.ports.length,
          big:event.data.cyclic.big.toString(), date:event.data.cyclic.date.getTime(),
          map:event.data.cyclic.map.get("key"), self:event.data.cyclic.self === event.data.cyclic,
          bytes:Array.from(new Uint8Array(event.data.cyclic.buffer))});
      };
      window.addEventListener("message",listener);
    });
    const ack = new Promise(resolve => {channel.port1.onmessage=event=>resolve(event.data);});
    const returned = win.postMessage({type:"window-message-probe",cyclic}, {transfer:[buffer,channel.port2]});
    const value = await received;
    const reply = await ack;
    channel.port1.close();
    let clone;
    try {structuredClone(win,{transfer:[win]});}catch(error){clone=error.name;}
    return {...value,reply,clone,detached:buffer.byteLength,returned:returned === undefined,id:id.toString()};
  });
  assert.equal(windowMessages.standard,true); assert.equal(windowMessages.trusted,true);
  assert.equal(windowMessages.source,null); assert.equal(windowMessages.ports,1);
  assert.equal(windowMessages.origin,appOrigin);
  assert.equal(windowMessages.big,"42"); assert.equal(windowMessages.date,0);
  assert.equal(windowMessages.map,"value"); assert.equal(windowMessages.self,true);
  assert.deepEqual(windowMessages.bytes,[1,2,3]); assert.equal(windowMessages.detached,0);
  assert.equal(windowMessages.reply,"ack"); assert.equal(windowMessages.clone,"DataCloneError");
  assert.equal(windowMessages.returned,true);
  const originFailure = await app.evaluate(async () => {
    const handle = await window.transferDemo.active.detach();
    const win = await (await import('/tuffite.js')).window.current();
    let invalid;
    try {win.postMessage({handle},{targetOrigin:"not-an-origin",transfer:[handle]});}catch(error){invalid=error.name;}
    const retained = handle.state;
    win.postMessage({handle},{targetOrigin:"https://unrelated.example",transfer:[handle]});
    return {invalid,retained,consumed:handle.state};
  });
  assert.equal(originFailure.invalid,"SyntaxError");
  assert.equal(originFailure.retained,"pending");
  assert.equal(originFailure.consumed,"transferred");
  await until(async()=>await targetCount()===0,"Dropped origin-mismatched message retained its page");
  await app.evaluate(() => window.transferDemo.load());
  console.log("handle phase: window MessageEvent, structured data, ports and origin checks passed");
  const late=await app.evaluate(() => window.transferDemo.popout());
  assert.equal(late.before.state.documentId,late.after.state.documentId);
  assert.equal(late.before.state.rendererProcess.id,late.after.state.rendererProcess.id);
  assert.equal(late.before.marker,late.after.marker);
  assert.equal(late.before.draft,late.after.draft);
  assert.equal(late.before.historyLength,late.after.historyLength);
  let receivedPage;
  await until(()=> (receivedPage=pages().find(page=>page.url().includes("receive="))),"Receiver missing");
  const message = await receivedPage.evaluate(()=>window.lastTransferMessage);
  assert.equal(message.standard,true); assert.equal(message.trusted,true);
  assert.equal(message.source,null); assert.equal(message.ports,1);
  assert.equal(message.origin,appOrigin);
  await app.goto(new URL("transfer.html?sender-left",base).href);
  let receiver;
  await until(()=> (receiver=pages().find(page=>page.url().includes("receive="))),"Transferred receiver missing");
  assert.equal((await receiver.evaluate(async ()=>window.transferDemo.snapshot())).state.documentId,late.after.state.documentId);
  await receiver.evaluate(async ()=>(await import('/tuffite.js')).window.close());
  await until(async()=> await targetCount()===0,"Closing the receiver retained its page");
  console.log(JSON.stringify({explorerHandle:"ok",stressTransfers:25,stressMs,exclusive:true,structuredTransfer:true,replaceRollback:true}));
} finally { await browser.close(); }

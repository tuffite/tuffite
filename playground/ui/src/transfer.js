import * as frameworkApi from "/tuffite.js";
// Only App top-level documents expose the native window bridge. The transfer
// lab also runs inside an ordinary iframe to exercise document ownership.
if (window.top === window) await import("/tuffite.js");

const status = document.querySelector("#status");
const hosts = document.querySelector("#hosts");
const receive = new URLSearchParams(location.search).get("receive");
let active;
let transfer;
let before;
const events = [];
const create = () => {
  const element = document.createElement("explorer");
  element.navigation = "allow";
  element.dialogs = "emit";
  element.contextMenuEnabled = true;
  element.style.cssText =
    "display:block;flex:1;min-width:0;height:100%;border:1px solid #888";
  for (const name of [
    "attached",
    "detached",
    "loadstart",
    "loadcommit",
    "loadstop",
  ])
    element.addEventListener(name, () => events.push(name));
  hosts.append(element);
  return element;
};
const first = create();
const second = create();
active = first;
const report = (value) => {
  status.textContent = JSON.stringify(value, null, 2);
};
const snapshot = async (element = active) => ({
  state: await element.getNavigationState(),
  ...(await element.executeJavaScript(
    "({marker: globalThis.transferMarker, draft: document.querySelector('#draft')?.value, historyLength: history.length})",
  )),
});
async function load() {
  if (transfer?.state === "pending") await transfer.close();
  transfer = undefined;
  const loaded = new Promise((resolve, reject) => {
    active.addEventListener("loadstop", resolve, { once: true });
    active.addEventListener("error", reject, { once: true });
  });
  active.load(new URL("./transfer-sample.html", document.baseURI).href);
  await loaded;
  await active.executeJavaScript(
    "globalThis.transferMarker = 'unsaved-' + Math.random(); document.querySelector('#draft').value = 'unsaved form input'; history.pushState({}, '', '?edited')",
  );
  before = await snapshot();
  report(before);
  return before;
}
async function detach() {
  before = await snapshot();
  transfer = await active.detach();
  const handle = transfer;
  handle.addEventListener("statechange", () =>
    report({ transfer: handle.state, events }),
  );
  report({ transfer: transfer.state, before });
  return transfer.state;
}
async function attach() {
  const target = active === first ? second : first;
  await target.attach(transfer);
  active = target;
  const after = await snapshot();
  const preserved =
    before.state.documentId === after.state.documentId &&
    before.marker === after.marker;
  report({ preserved, after, events });
  if (!preserved) throw new Error("Page state changed during transfer");
  return after;
}
async function close() {
  await transfer?.close();
  report({ transfer: transfer?.state, events });
}
async function popout() {
  const name = crypto.randomUUID();
  const channel = new BroadcastChannel(`explorer-transfer:${name}`);
  const destination = new URL(location.href);
  destination.search = new URLSearchParams({ receive: name }).toString();
  let timer, child;
  const ack = new MessageChannel();
  try {
    const ready = new Promise((resolve, reject) => {
      timer = setTimeout(() => reject(new Error("Target window did not become ready")), 15000);
      channel.onmessage = ({data}) => { if (data.type === "ready") resolve(); };
    });
    void ready.catch(() => {});
    child = await frameworkApi.window.create({
      url: destination.href, title: "Transferred Explorer",
      bounds: { x: 100, y: 100, width: 900, height: 600 },
    });
    await ready; clearTimeout(timer); channel.close();
    await detach();
    const result = new Promise((resolve, reject) => {
      timer = setTimeout(() => reject(new Error("Target window did not attach")), 15000);
      ack.port1.onmessage = ({data}) => {
        if (data.ok) resolve(data.snapshot); else reject(new Error(data.error));
      };
    });
    child.postMessage({type: "attach-explorer", session: name, handle: transfer}, {
      targetOrigin: destination.origin, transfer: [transfer, ack.port2],
    });
    const after = await result;
    report({independent:true, preserved:before.state.documentId === after.state.documentId, after});
    return { childId: child.id.toString(), before, after };
  } catch (error) {
    if (transfer?.state === "pending") await recover();
    if (child) await frameworkApi.window.close(child.id);
    throw error;
  } finally { clearTimeout(timer); channel.close(); ack.port1.close(); ack.port2.close(); }
}
async function recover() {
  const handle = transfer;
  if (!handle) return false;
  let generation = 0;
  let wake;
  const changed = () => {
    generation++;
    wake?.();
    wake = undefined;
  };
  const wait = (since) =>
    generation !== since
      ? Promise.resolve()
      : new Promise((resolve) => {
          wake = resolve;
        });
  handle.addEventListener("statechange", changed);
  try {
    for (;;) {
      if (handle.state === "attaching") {
        await wait(generation);
        continue;
      }
      if (handle.state === "transferred") return true;
      if (handle.state === "closed") return false;
      if (active.isConnected) {
        try {
          await active.attach(handle);
          return false;
        } catch {}
      }
      const since = generation;
      try {
        await handle.close();
        return handle.state === "transferred";
      } catch (error) {
        if (handle.state === "transferred") return true;
        if (handle.state === "closed") return false;
        if (error.name !== "InvalidStateError") throw error;
        await wait(since);
      }
    }
  } finally {
    handle.removeEventListener("statechange", changed);
  }
}
window.transferDemo = {
  first,
  second,
  get active() {
    return active;
  },
  get transfer() {
    return transfer;
  },
  events,
  load,
  detach,
  attach,
  close,
  snapshot,
  popout,
};
for (const [id, action] of Object.entries({
  load,
  detach,
  attach,
  close,
  popout,
}))
  document.querySelector(`#${id}`).onclick = () =>
    action().catch((error) => report({ error: error.message }));
if (receive) {
  document.querySelector("nav").hidden = true;
  second.remove();
  window.addEventListener("message", async event => {
    if (event.origin !== location.origin || event.data?.type !== "attach-explorer" ||
        event.data.session !== receive) return;
    const handle = event.data.handle, port = event.ports[0];
    window.lastTransferMessage = {standard:event instanceof MessageEvent, trusted:event.isTrusted,
      origin:event.origin, source:event.source, ports:event.ports.length};
    try {
      await first.attach(handle);
      const result = await snapshot(first);
      report(result); port.postMessage({ok:true, snapshot:result});
    } catch(error) {
      await handle.close(); port.postMessage({ok:false, error:error.message});
    } finally { port.close(); }
  });
  const channel = new BroadcastChannel(`explorer-transfer:${receive}`);
  channel.postMessage({type:"ready"}); channel.close();
}

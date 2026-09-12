// Only App top-level documents expose the native window bridge. The transfer
// lab also runs inside an ordinary iframe to exercise document ownership.
if (window.top === window) await import("/shell_api.js");

const status = document.querySelector("#status");
const hosts = document.querySelector("#hosts");
const receive = new URLSearchParams(location.search).get("receive");
let active;
let transfer;
let before;
const events = [];
const create = () => {
  const element = document.createElement("explorer");
  element.deferred = true;
  element.navigation = "allow";
  element.dialogs = "emit";
  element.contextMenuEnabled = true;
  element.style.cssText = "display:block;flex:1;min-width:0;height:100%;border:1px solid #888";
  for (const name of ["attached", "detached", "loadstart", "loadcommit", "loadstop"])
    element.addEventListener(name, () => events.push(name));
  hosts.append(element);
  return element;
};
const first = create();
const second = create();
active = first;
const report = (value) => { status.textContent = JSON.stringify(value, null, 2); };
const snapshot = async (element = active) => ({
  state: await element.getNavigationState(),
  ...await element.executeJavaScript("({marker: globalThis.transferMarker, draft: document.querySelector('#draft')?.value, historyLength: history.length})"),
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
  await active.executeJavaScript("globalThis.transferMarker = 'unsaved-' + Math.random(); document.querySelector('#draft').value = 'unsaved form input'; history.pushState({}, '', '?edited')");
  before = await snapshot();
  report(before);
  return before;
}
async function detach() {
  before = await snapshot();
  transfer = await active.detach();
  const handle = transfer;
  handle.addEventListener("statechange", () => report({ transfer: handle.state, events }));
  report({ transfer: transfer.state, before });
  return transfer.token;
}
async function attach() {
  const target = active === first ? second : first;
  await target.attach(transfer.token);
  active = target;
  const after = await snapshot();
  const preserved = before.state.documentId === after.state.documentId && before.marker === after.marker;
  report({ preserved, after, events });
  if (!preserved) throw new Error("Page state changed during transfer");
  return after;
}
async function close() {
  await transfer?.close();
  report({ transfer: transfer?.state, events });
}
async function popout() {
  const channelId = crypto.randomUUID();
  const channel = new BroadcastChannel(`explorer-transfer:${channelId}`);
  const destination = new URL(location.href);
  destination.search = new URLSearchParams({ receive: channelId }).toString();
  let timer;
  let child;
  let cancelled = false;
  let started = false;
  try {
    const result = new Promise((resolve, reject) => {
      timer = setTimeout(() => reject(new Error("Target window did not complete transfer")), 15000);
      channel.onmessage = async ({ data }) => {
        try {
          if (data.type === "ready" && !started && !cancelled) {
            started = true;
            const token = await detach();
            if (cancelled) await recover();
            else channel.postMessage({ type: "attach", token });
          }
          if (data.type === "attached") resolve(data.snapshot);
          if (data.type === "error") reject(new Error(data.error));
        } catch (error) { reject(error); }
      };
    });
    void result.catch(() => {});
    child = await window.tuffite.window.create({ url: destination.href, title: "Transferred Explorer", bounds: { x: 100, y: 100, width: 900, height: 600 } });
    const after = await result;
    report({ independent: true, preserved: before.state.documentId === after.state.documentId && before.marker === after.marker, after });
    return { childId: child.id.toString(), before, after };
  } catch (error) {
    cancelled = true;
    const committed = await recover();
    if (child && !committed) channel.postMessage({ type: "cancel" });
    throw error;
  } finally {
    clearTimeout(timer);
    channel.close();
  }
}
async function recover() {
  const handle = transfer;
  if (!handle) return false;
  let generation = 0;
  let wake;
  const changed = () => { generation++; wake?.(); wake = undefined; };
  const wait = (since) => generation !== since ? Promise.resolve() : new Promise((resolve) => { wake = resolve; });
  handle.addEventListener("statechange", changed);
  try {
    for (;;) {
      if (handle.state === "attaching") { await wait(generation); continue; }
      if (handle.state === "transferred") return true;
      if (handle.state === "closed") return false;
      if (active.isConnected) {
        try { await active.attach(handle.token); return false; } catch {}
      }
      const since = generation;
      try { await handle.close(); return handle.state === "transferred"; }
      catch (error) {
        if (handle.state === "transferred") return true;
        if (handle.state === "closed") return false;
        if (error.name !== "InvalidStateError") throw error;
        await wait(since);
      }
    }
  } finally { handle.removeEventListener("statechange", changed); }
}
window.transferDemo = { first, second, get active() { return active; }, get transfer() { return transfer; }, events, load, detach, attach, close, snapshot, popout };
for (const [id, action] of Object.entries({ load, detach, attach, close, popout }))
  document.querySelector(`#${id}`).onclick = () => action().catch((error) => report({ error: error.message }));
if (receive) {
  document.querySelector("nav").hidden = true;
  second.remove();
  const channel = new BroadcastChannel(`explorer-transfer:${receive}`);
  let attaching = false;
  let expired = false;
  const closeEmpty = () => {
    if (!attaching && first.deferred) void window.tuffite.window.close();
  };
  const timer = setTimeout(() => { expired = true; closeEmpty(); }, 15000);
  channel.onmessage = async ({ data }) => {
    if (data.type === "cancel") { closeEmpty(); return; }
    if (data.type !== "attach" || attaching) return;
    attaching = true;
    try {
      await first.attach(data.token);
      const result = await snapshot(first);
      report(result);
      channel.postMessage({ type: "attached", snapshot: result });
    } catch (error) { channel.postMessage({ type: "error", error: error.message }); }
    finally {
      attaching = false;
      if (!first.deferred) { clearTimeout(timer); channel.close(); }
      else if (expired) closeEmpty();
    }
  };
  channel.postMessage({ type: "ready" });
}

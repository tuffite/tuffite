import "/shell_api.js";
import { setupUtilityLifecycle } from "./utility-lifecycle.js";
import { setupWindowManagement } from "./window-management.js";

const runtime = document.querySelector("#runtime");
const networkOutput = document.querySelector("#network");
const explorerSlot = document.querySelector("#thread-explorer-slot");
const isolatedExplorerSlot = document.querySelector("#isolated-explorer-slot");
const persistentExplorerSlot = document.querySelector(
  "#persistent-explorer-slot",
);
const inheritPersistentExplorerSlot = document.querySelector(
  "#inherit-persistent-explorer-slot",
);
const inheritMemoryExplorerSlot = document.querySelector(
  "#inherit-memory-explorer-slot",
);
let explorer = createExplorer();
const partitionExplorers = new Map();
const nestedStorageProofs = new Map();
const nestedStorageWaiters = new Map();
const nestedBubbleWaiters = new Map();
let nestedWindow;
const explorerV2Output = document.querySelector("#explorer-v2");
const explorerSuiteStatus = document.querySelector("#explorer-suite-status");
const explorerCommandResult = document.querySelector(
  "#explorer-command-result",
);
const explorerActionStatus = document.querySelector("#explorer-action-status");
const explorerDialogResult = document.querySelector("#explorer-dialog-result");
const explorerStorageResult = document.querySelector(
  "#explorer-storage-result",
);
const explorerEvents = document.querySelector("#explorer-events");
const explorerNotification = document.querySelector("#explorer-notification");
const button = document.querySelector("#invoke");
const dialogOutput = document.querySelector("#dialog");
const pageOptions = new URLSearchParams(window.location.search);
const explorerOnlySmoke = pageOptions.has("explorer-smoke");
// A JavaScript dialog blocks its renderer thread. In --single-process smoke
// runs the parent cannot receive and answer the child's dialog event until the
// same thread unblocks, so keep this probe opt-in. The interactive controls
// remain available in normal multiprocess runs.
const checkSynchronousDialog = pageOptions.has("dialog-check");
// An independent partition needs its own renderer process. Keep it lazy for
// the interactive page, while a multiprocess runner can include it at startup.
const checkIndependentPartitionAtStartup = pageOptions.has("partition-check");

const ready = {
  runtime: explorerOnlySmoke,
  network: explorerOnlySmoke,
  platform: explorerOnlySmoke,
  threadPool: explorerOnlySmoke,
  explorer: false,
  explorerV2: false,
  utility: explorerOnlySmoke,
  utilityConcurrent: explorerOnlySmoke,
  utilitySession: explorerOnlySmoke,
  javascriptExtension: explorerOnlySmoke,
  largeAsset: explorerOnlySmoke,
  applicationEvent: explorerOnlySmoke,
  window: explorerOnlySmoke,
};

const explorerChecks = new Set();
let dialogResolved = false;
const openEventsSeen = new Set();
let pendingDialog;
let explorerSuiteFinishing = false;
let explorerSuiteComplete = false;

function createExplorer() {
  const element = document.createElement("explorer");
  element.id = "thread-explorer";
  element.navigation = "emit";
  element.dialogs = "emit";
  element.contextMenuEnabled =
    document.querySelector("#explorer-context-menu")?.checked ?? true;
  element.src = new URL("./explorer.html?mode=thread", document.baseURI).href;
  bindExplorerEvents(element);
  return element;
}

function trackPendingDialog(dialog) {
  pendingDialog = dialog;
  dialog.signal.addEventListener(
    "abort",
    () => {
      if (pendingDialog !== dialog) return;
      pendingDialog = undefined;
      setDialogButtons(false);
      explorerDialogResult.textContent = `Cancelled ${dialog.type}: source or handler exited`;
      logExplorerEvent("dialog:cancelled", {
        type: dialog.type,
        from: dialog.from,
      });
    },
    { once: true },
  );
}

function bindExplorerEvents(element) {
  for (const name of ["loadstart", "loadstop", "loaderror", "loadcommit"]) {
    element.addEventListener(name, (event) =>
      logExplorerEvent(name, event.detail),
    );
  }
  for (const name of ["navigation", "newwindow"]) {
    element.addEventListener(name, (event) => {
      openEventsSeen.add(name);
      logExplorerEvent(name, {
        url: event.detail.url,
        disposition: event.detail.disposition,
        userGesture: event.detail.userGesture,
        from: event.detail.from,
      });
      markExplorerCheck(`${name}-event`);
      if (openEventsSeen.size === 2) markExplorerCheck("open-events");
    });
  }
  element.addEventListener("dialog", async (event) => {
    // Leave the nested fixture's request unclaimed so the top fallback can
    // demonstrate handling a request after both Explorer boundaries.
    if (event.detail.message === "Dialog emitted by the nested Explorer") return;
    event.preventDefault();
    const dialog = event.detail;
    trackPendingDialog(dialog);
    explorerDialogResult.textContent = formatValue({
      type: dialog.type,
      message: dialog.message,
      defaultValue: dialog.defaultValue,
      isReload: dialog.isReload,
    });
    setDialogButtons(true);
    logExplorerEvent(`dialog:${dialog.type}`, {
      type: dialog.type,
      message: dialog.message,
      defaultValue: dialog.defaultValue,
      isReload: dialog.isReload,
    });
    if (document.querySelector("#explorer-dialog-mode").value !== "auto")
      return;
    const accepted = await dialog.accept(
      document.querySelector("#explorer-dialog-text").value,
    );
    pendingDialog = undefined;
    setDialogButtons(false);
    explorerDialogResult.textContent = `Automatically accepted ${dialog.type}: ${accepted}`;
    if (dialog.type === "confirm") {
      dialogResolved = accepted;
      if (accepted) markExplorerCheck("dialog");
    }
  });
}

function logExplorerEvent(type, detail) {
  const item = document.createElement("li");
  const time = new Date().toLocaleTimeString([], { hour12: false });
  item.textContent = `${time} ${type}${detail ? ` ${formatValue(detail)}` : ""}`;
  explorerEvents.prepend(item);
  while (explorerEvents.children.length > 80)
    explorerEvents.lastElementChild.remove();
}

function formatValue(value) {
  return JSON.stringify(value, null, 2);
}

function setExplorerActionStatus(message, state = "") {
  explorerActionStatus.className = `action-status${state ? ` ${state}` : ""}`;
  explorerActionStatus.textContent = message;
}

function setDialogButtons(enabled) {
  document.querySelector("#explorer-dialog-accept").disabled = !enabled;
  document.querySelector("#explorer-dialog-reject").disabled = !enabled;
}

function reportExplorerStage(stage) {
  document.title = `Tuffite Explorer Stage: ${stage}`;
}

function markExplorerCheck(name) {
  explorerChecks.add(name);
  explorerV2Output.textContent = `Passed: ${[...explorerChecks].sort().join(" · ")}`;
  if (document.title.startsWith("Tuffite Explorer Stage: waiting")) {
    reportExplorerStage(`waiting:${[...explorerChecks].sort().join(",")}`);
  }
  const required = [
    ...(checkSynchronousDialog ? ["dialog"] : []),
    "executeJavaScript",
    "history",
    "shared-partition",
    ...(checkIndependentPartitionAtStartup ? ["isolated-partition"] : []),
    "navigation-event",
    "nested-message",
    "open-events",
    "parent-message",
  ];
  if (required.every((name) => explorerChecks.has(name))) {
    void finishExplorerSuite();
  }
}

async function finishExplorerSuite() {
  if (explorerSuiteFinishing || explorerSuiteComplete) return;
  explorerSuiteFinishing = true;
  try {
    await waitForExplorerLoad(explorer, () => {
      explorer.load(
        new URL("./explorer.html?mode=complete", document.baseURI).href,
      );
    });
    explorerSuiteComplete = true;
    explorerSuiteStatus.className = "status-pill ready";
    explorerSuiteStatus.textContent = checkIndependentPartitionAtStartup
      ? "All checks passed"
      : "Core checks passed";
    reportReady("explorerV2");
    logExplorerEvent("self-check:complete", {
      checks: [...explorerChecks].sort(),
    });
  } catch (error) {
    reportError("explorer v2", error);
  } finally {
    explorerSuiteFinishing = false;
  }
}

window.addEventListener("message", async (event) => {
  const { data, source } = event;
  if (event.origin !== location.origin) return;
  const from = source === explorer.contentWindow ? "child" : "nested";
  if (data?.type === "explorer-ready" && data.storage === "shared-from-app") {
    markExplorerCheck("shared-partition");
    return;
  }
  if (data?.type === "memory-ready" && data.storage === null) {
    markExplorerCheck("isolated-partition");
    return;
  }
  if (data?.type === "nested-storage-ready") {
    nestedStorageProofs.set(data.ownerMode, data);
    nestedStorageWaiters.get(data.ownerMode)?.(data);
    nestedStorageWaiters.delete(data.ownerMode);
    logExplorerEvent(`partition:nested-shared-${data.ownerMode}`, data);
    return;
  }
  if (data?.type === "parent-message-ack") {
    markExplorerCheck("parent-message");
    explorerCommandResult.textContent = `Child acknowledged: ${formatValue(data.echo)}`;
    logExplorerEvent("surface:child-to-top", { from, data });
    return;
  }
  if (data?.type === "notification-request") {
    nestedWindow = source;
    explorerNotification.hidden = false;
    explorerNotification.textContent = `${data.notification.title}: ${data.notification.body}`;
    setTimeout(() => (explorerNotification.hidden = true), 3500);
    logExplorerEvent("message:notification-request", { from, data });
    source.postMessage(
      {
        type: "notification-result",
        requestId: data.requestId,
        shown: true,
      },
      { targetOrigin: event.origin },
    );
    return;
  }
  if (data?.type === "nested-message-complete" && data.shown === true) {
    markExplorerCheck("nested-message");
    logExplorerEvent("surface:nested-message-complete", { from, data });
    return;
  }
  if (data?.type === "nested-bubble-hop") {
    logExplorerEvent(`${data.layer}:${data.eventType}`, {
      from,
      stopped: data.stopped,
    });
    return;
  }
  if (data?.type === "nested-request-triggered") {
    logExplorerEvent(`level-2-trigger:${data.eventType}`, { from });
    return;
  }
  if (data?.type === "nested-bubble-configured") {
    nestedBubbleWaiters.get(data.requestId)?.();
    nestedBubbleWaiters.delete(data.requestId);
    return;
  }
  if (data?.type === "nested-dialog-result") {
    logExplorerEvent("nested:dialog-result", { from, accepted: data.accepted });
  }
});

for (const type of ["navigation", "newwindow"]) {
  window.tuffite.addEventListener(type, (event) => {
    logExplorerEvent(`top:${type}`, event.detail);
  });
}
window.tuffite.addEventListener("dialog", (event) => {
  event.preventDefault();
  const dialog = event.detail;
  trackPendingDialog(dialog);
  setDialogButtons(true);
  explorerDialogResult.textContent = formatValue({
    layer: "top",
    type: dialog.type,
    message: dialog.message,
    defaultValue: dialog.defaultValue,
    isReload: dialog.isReload,
    from: dialog.from,
  });
  logExplorerEvent("top:dialog", {
    type: dialog.type,
    message: dialog.message,
    defaultValue: dialog.defaultValue,
    isReload: dialog.isReload,
    from: dialog.from,
  });
  if (document.querySelector("#explorer-dialog-mode").value === "auto") {
    void dialog
      .accept(document.querySelector("#explorer-dialog-text").value)
      .then((accepted) => {
        if (pendingDialog === dialog) {
          pendingDialog = undefined;
          setDialogButtons(false);
        }
        logExplorerEvent("top:dialog-response", {
          type: dialog.type,
          accepted,
        });
      });
  }
});

window.tuffite?.addEventListener("channel", ({ detail: { channel, data } }) => {
  const payload = new TextDecoder().decode(data);
  if (channel === "playground:command" && payload === "hello from Chromium") {
    reportReady("applicationEvent");
  } else if (
    channel === "playground:thread-pool" &&
    payload === "chromium-thread-pool"
  ) {
    reportReady("threadPool");
  } else if (channel === "playground:utility-session") {
    if (payload === "ok") reportReady("utilitySession");
    else
      reportError(
        "utility session",
        new Error("cross-process session validation failed"),
      );
  } else if (channel === "playground:javascript-extension") {
    if (payload === "ok") reportReady("javascriptExtension");
    else
      reportError(
        "JavaScript extension",
        new Error("restricted V8 extension validation failed"),
      );
  }
});

function reportReady(feature) {
  ready[feature] = true;
  if (Object.values(ready).every(Boolean)) {
    document.title = "Tuffite App Ready";
  } else {
    document.title = `Tuffite Ready: ${Object.entries(ready)
      .filter(([, value]) => value)
      .map(([name]) => name)
      .join(",")}`;
  }
}

function reportError(feature, error) {
  document.title = `Tuffite ${feature} Error: ${error.message}`;
  if (feature === "explorer v2") {
    explorerSuiteStatus.className = "status-pill failed";
    explorerSuiteStatus.textContent = "Self-check failed";
    explorerV2Output.textContent = `Error: ${error.message}`;
    logExplorerEvent("self-check:error", { message: error.message });
  }
}

const checkUtility = setupUtilityLifecycle({
  onReady: () => reportReady("utility"),
  onConcurrentReady: () => reportReady("utilityConcurrent"),
});
const checkWindowManagement = setupWindowManagement({
  onReady: () => reportReady("window"),
});

async function invoke() {
  try {
    const response = await window.playground.commands.doSomething(
      "hello from Chromium",
    );
    runtime.textContent = response;
    reportReady("runtime");
  } catch (error) {
    runtime.textContent = `ShellAPI error: ${error.message}`;
    reportError("runtime", error);
  }
}

async function checkNetwork() {
  try {
    try {
      await window.tuffite.net.request("http://localhost/");
      throw new Error("undeclared network host was allowed");
    } catch (error) {
      if (error.message === "undeclared network host was allowed") throw error;
    }
    const target = window.tuffite.networkTestUrl || "https://example.com/";
    const response = await window.tuffite.net.request(target);
    networkOutput.textContent = `HTTP ${response.status} · ${response.body.byteLength} bytes`;
    if (response.status >= 200 && response.status < 400) reportReady("network");
  } catch (error) {
    networkOutput.textContent = `Network error: ${error.message}`;
    reportError("network", error);
  }
}

async function checkPlatformApis() {
  const output = document.querySelector("#platform");
  try {
    const directories = await window.tuffite.path.directories();
    const dataFile = await window.tuffite.path.resolve(
      "data",
      "playground.bin",
    );
    const value = new TextEncoder().encode("persistent playground value");
    await window.tuffite.fs.writeFile("data", "playground.bin", value);
    const read = await window.tuffite.fs.readFile("data", "playground.bin");
    if (
      new TextDecoder().decode(read) !== "persistent playground value" ||
      !(await window.tuffite.fs.exists("data", "playground.bin"))
    ) {
      throw new Error("filesystem round trip failed");
    }
    await window.tuffite.storage.set("playground", value);
    const stored = await window.tuffite.storage.get("playground");
    if (
      !stored ||
      new TextDecoder().decode(stored) !== "persistent playground value"
    ) {
      throw new Error("storage round trip failed");
    }
    await window.tuffite.log.info("Playground platform API smoke passed");
    output.textContent = `data: ${directories.data}\nfile: ${dataFile}`;
    reportReady("platform");
  } catch (error) {
    output.textContent = `Platform API error: ${error.message}`;
    reportError("platform", error);
  }
}

async function checkLargeAsset() {
  const output = document.querySelector("#async-framework");
  try {
    const response = await fetch("./playground-public/large-asset.txt");
    const body = new Uint8Array(await response.arrayBuffer());
    if (
      !response.ok ||
      body.byteLength < 256 * 1024 ||
      body.subarray(0, 8).some((byte) => byte !== 120)
    ) {
      throw new Error(
        `unexpected large asset response: ${response.status}, ${body.byteLength} bytes`,
      );
    }
    output.textContent = `Large packaged asset: ${body.byteLength} bytes`;
    reportReady("largeAsset");
  } catch (error) {
    output.textContent = `Large asset error: ${error.message}`;
    reportError("large asset", error);
  }
}

button.addEventListener("click", invoke);
document.querySelector("#dialog-file").addEventListener("click", async () => {
  const selected = await window.tuffite.dialog.openFile({
    title: "Choose a Playground file",
    extensions: [["html", "md"]],
  });
  dialogOutput.textContent = selected ?? "File selection cancelled";
});
document
  .querySelector("#dialog-directory")
  .addEventListener("click", async () => {
    const selected = await window.tuffite.dialog.openDirectory({
      title: "Choose a Playground directory",
    });
    dialogOutput.textContent = selected ?? "Directory selection cancelled";
  });
function waitForExplorerLoad(element, action, timeoutMs = 15000) {
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      clearTimeout(timeout);
      element.removeEventListener("loadstop", loaded);
      element.removeEventListener("loaderror", failed);
      element.removeEventListener("error", failed);
    };
    const loaded = (event) => {
      cleanup();
      resolve(event);
    };
    const failed = (event) => {
      cleanup();
      reject(
        new Error(
          `Explorer load was rejected${event.detail ? `: ${formatValue(event.detail)}` : ""}`,
        ),
      );
    };
    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error("Explorer load timed out"));
    }, timeoutMs);
    element.addEventListener("loadstop", loaded);
    element.addEventListener("loaderror", failed);
    element.addEventListener("error", failed);
    action();
  });
}

let explorerCommandRunning = false;
async function runExplorerCommand(label, operation) {
  if (explorerCommandRunning) {
    // Keep Stop available while a navigation is waiting for completion.
    if (label === "stop") {
      try {
        return await operation();
      } catch (error) {
        setExplorerActionStatus(`Stop failed: ${error.message}`, "failed");
      }
    }
    return;
  }
  explorerCommandRunning = true;
  const controls = [
    ...document.querySelectorAll(
      ".explorer-toolbar button:not(#explorer-stop), #explorer-execute, #explorer-state, #explorer-message, #explorer-recreate, #explorer-stress",
    ),
  ];
  const disabled = controls.map((control) => control.disabled);
  controls.forEach((control) => {
    control.disabled = true;
  });
  explorerCommandResult.setAttribute("aria-busy", "true");
  setExplorerActionStatus(`${label} …`, "running");
  explorerCommandResult.textContent = `${label}: running…`;
  try {
    const result = await operation();
    explorerCommandResult.textContent = `${label}: ${formatValue(result)}`;
    setExplorerActionStatus(`${label} completed`, "success");
    logExplorerEvent(`api:${label}`, result);
    return result;
  } catch (error) {
    explorerCommandResult.textContent = `${label} failed: ${error.message}`;
    setExplorerActionStatus(`${label} failed: ${error.message}`, "failed");
    logExplorerEvent(`api:${label}:error`, { message: error.message });
    return undefined;
  } finally {
    controls.forEach((control, index) => {
      control.disabled = disabled[index];
    });
    explorerCommandResult.setAttribute("aria-busy", "false");
    explorerCommandRunning = false;
  }
}

function loadExplorer(label, url) {
  return runExplorerCommand(label, async () => {
    await waitForExplorerLoad(explorer, () => explorer.load(url), 45000);
    return explorer.getNavigationState();
  });
}

async function executeInExplorer(source) {
  const state = await explorer.getNavigationState();
  return explorer.executeJavaScript(source, {
    expectedDocumentId: state.documentId,
  });
}

async function triggerDialog(kind) {
  const source = {
    alert: `alert('Alert emitted by the Explorer'); 'alert returned'`,
    confirm: `confirm('Accept this Explorer confirm?')`,
    prompt: `prompt('Value returned by parent', 'Explorer default')`,
    beforeunload: `window.onbeforeunload = () => 'Unsaved Explorer work'; location.reload(); true`,
  }[kind];
  return runExplorerCommand(`dialog.${kind}`, () => executeInExplorer(source));
}

async function answerPendingDialog(accept) {
  if (!pendingDialog) return;
  const dialog = pendingDialog;
  pendingDialog = undefined;
  setDialogButtons(false);
  const accepted = accept
    ? await dialog.accept(document.querySelector("#explorer-dialog-text").value)
    : await dialog.dismiss();
  explorerDialogResult.textContent = `${accept ? "Accepted" : "Cancelled"} ${dialog.type}: ${accepted}`;
  logExplorerEvent("dialog:response", { type: dialog.type, accept, accepted });
}

async function ensurePartitionExplorer(kind) {
  const configurations = {
    persist: {
      partition: "persist:playground-v2-persist",
      mode: "persist",
      slot: persistentExplorerSlot,
    },
    memory: {
      partition: "memory:playground-v2-memory",
      mode: "isolated",
      slot: isolatedExplorerSlot,
    },
    "inherit+persist": {
      partition: "inherit+persist:playground-v2-inherit-persist",
      mode: "inherit-persist",
      slot: inheritPersistentExplorerSlot,
    },
    "inherit+memory": {
      partition: "inherit+memory:playground-v2-inherit-memory",
      mode: "inherit-memory",
      slot: inheritMemoryExplorerSlot,
    },
  };
  const configuration = configurations[kind];
  if (!configuration) throw new TypeError(`Unknown partition mode: ${kind}`);
  const existing = partitionExplorers.get(kind);
  if (existing?.isConnected) return existing;
  const element = document.createElement("explorer");
  element.id = `${kind.replaceAll("+", "-")}-explorer`;
  element.className = "compact-explorer";
  element.partition = configuration.partition;
  element.navigation = "emit";
  element.dialogs = "suppress";
  element.src = new URL(
    `./explorer.html?mode=${configuration.mode}`,
    document.baseURI,
  ).href;
  for (const name of ["loadstart", "loadstop", "loaderror", "loadcommit"]) {
    element.addEventListener(name, (event) =>
      logExplorerEvent(`${kind}:${name}`, event.detail),
    );
  }
  partitionExplorers.set(kind, element);
  await waitForExplorerLoad(element, () =>
    configuration.slot.replaceChildren(element),
  );
  return element;
}

const ensureIsolatedExplorer = () => ensurePartitionExplorer("memory");

function waitForNestedStorageProof(mode, timeoutMs = 15000) {
  if (nestedStorageProofs.has(mode)) {
    return Promise.resolve(nestedStorageProofs.get(mode));
  }
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      nestedStorageWaiters.delete(mode);
      reject(new Error(`Nested ${mode} shared-partition probe timed out`));
    }, timeoutMs);
    nestedStorageWaiters.set(mode, (value) => {
      clearTimeout(timeout);
      resolve(value);
    });
  });
}

function recreateExplorer(onAppend) {
  const previous = explorer;
  explorer = createExplorer();
  const completion = onAppend?.(explorer);
  explorerSlot.replaceChildren(explorer);
  previous.remove();
  explorerCommandResult.textContent = "Recreated the normal inner WebContents.";
  setExplorerActionStatus("Recreate completed", "success");
  logExplorerEvent("lifecycle:recreate");
  return completion;
}

async function stressExplorer() {
  const button = document.querySelector("#explorer-stress");
  button.disabled = true;
  try {
    for (let index = 1; index <= 20; index++) {
      await recreateExplorer((next) =>
        waitForExplorerLoad(next, () => {}, 10000),
      );
      const state = await explorer.getNavigationState();
      if (!state.documentId)
        throw new Error(`iteration ${index} has no documentId`);
      explorerCommandResult.textContent = `Stress ${index}/20 · ${state.url}`;
    }
    explorerCommandResult.textContent =
      "Stress ×20 completed; all WebContents loaded.";
    logExplorerEvent("lifecycle:stress-complete", { iterations: 20 });
  } finally {
    button.disabled = false;
  }
}

document
  .querySelector("#explorer-home")
  .addEventListener(
    "click",
    () =>
      void loadExplorer(
        "load(Home)",
        new URL("./explorer.html?mode=manual", document.baseURI).href,
      ),
  );
document.querySelector("#explorer-load-url").addEventListener("click", () => {
  const input = document.querySelector("#explorer-url");
  void loadExplorer("load(URL)", input.value.trim());
});
document.querySelector("#explorer-url").addEventListener("keydown", (event) => {
  if (event.key !== "Enter") return;
  event.preventDefault();
  document.querySelector("#explorer-load-url").click();
});
document
  .querySelector("#explorer-history-a")
  .addEventListener(
    "click",
    () =>
      void loadExplorer(
        "load(A)",
        new URL("./history-a.html", document.baseURI).href,
      ),
  );
document
  .querySelector("#explorer-history-b")
  .addEventListener(
    "click",
    () =>
      void loadExplorer(
        "load(B)",
        new URL("./history-b.html", document.baseURI).href,
      ),
  );
document
  .querySelector("#explorer-baidu")
  .addEventListener(
    "click",
    () => void loadExplorer("load(Baidu)", "https://www.baidu.com/"),
  );
document
  .querySelector("#explorer-back")
  .addEventListener(
    "click",
    () => void runExplorerCommand("back", () => explorer.back()),
  );
document
  .querySelector("#explorer-forward")
  .addEventListener(
    "click",
    () => void runExplorerCommand("forward", () => explorer.forward()),
  );
document
  .querySelector("#explorer-go-back")
  .addEventListener(
    "click",
    () => void runExplorerCommand("go(-1)", () => explorer.go(-1)),
  );
document
  .querySelector("#explorer-reload")
  .addEventListener(
    "click",
    () => void runExplorerCommand("reload", () => explorer.reload()),
  );
document
  .querySelector("#explorer-reload-cache")
  .addEventListener(
    "click",
    () =>
      void runExplorerCommand("reload(bypassCache)", () =>
        explorer.reload({ bypassCache: true }),
      ),
  );
document
  .querySelector("#explorer-stop")
  .addEventListener(
    "click",
    () => void runExplorerCommand("stop", () => explorer.stop()),
  );
document
  .querySelector("#explorer-execute")
  .addEventListener(
    "click",
    () =>
      void runExplorerCommand("executeJavaScript", () =>
        executeInExplorer(
          `({title: document.title, href: location.href, now: new Date().toISOString()})`,
        ),
      ),
  );
document
  .querySelector("#explorer-state")
  .addEventListener(
    "click",
    () =>
      void runExplorerCommand("getNavigationState", () =>
        explorer.getNavigationState(),
      ),
  );
document.querySelector("#explorer-message").addEventListener(
  "click",
  () =>
    void runExplorerCommand("postMessage", () =>
      explorer.postMessage({
        type: "parent-probe",
        payload: { sentAt: new Date().toISOString() },
      }),
    ),
);
document
  .querySelector("#explorer-recreate")
  .addEventListener("click", () => recreateExplorer());
document
  .querySelector("#explorer-stress")
  .addEventListener("click", () => void stressExplorer());
document
  .querySelector("#explorer-context-menu")
  .addEventListener("change", (event) => {
    explorer.contextMenuEnabled = event.currentTarget.checked;
    const state = event.currentTarget.checked ? "enabled" : "disabled";
    explorerCommandResult.textContent = `contextMenuEnabled: ${event.currentTarget.checked}`;
    setExplorerActionStatus(`Native context menu ${state}`, "success");
    logExplorerEvent("attribute:contextMenuEnabled", {
      enabled: event.currentTarget.checked,
    });
  });
for (const kind of ["alert", "confirm", "prompt", "beforeunload"]) {
  document
    .querySelector(`#explorer-${kind}`)
    .addEventListener("click", () => void triggerDialog(kind));
}
document
  .querySelector("#explorer-dialog-accept")
  .addEventListener("click", () => void answerPendingDialog(true));
document
  .querySelector("#explorer-dialog-reject")
  .addEventListener("click", () => void answerPendingDialog(false));
document
  .querySelector("#explorer-storage")
  .addEventListener("click", async () => {
    try {
      explorerStorageResult.textContent =
        "Creating all independent and inherited partitions…";
      const modes = ["persist", "memory", "inherit+persist", "inherit+memory"];
      const explorers = Object.fromEntries(
        await Promise.all(
          modes.map(async (mode) => [
            mode,
            await ensurePartitionExplorer(mode),
          ]),
        ),
      );
      const sharedState = await explorer.getNavigationState();
      const states = Object.fromEntries(
        await Promise.all(
          modes.map(async (mode) => [
            mode,
            await explorers[mode].getNavigationState(),
          ]),
        ),
      );
      const children = Object.fromEntries(
        await Promise.all(
          modes.map(async (mode) => [
            mode,
            await waitForNestedStorageProof(mode),
          ]),
        ),
      );
      const sharedStorage = await explorer.executeJavaScript(
        `localStorage.getItem('explorer-v2-storage')`,
      );
      const storage = Object.fromEntries(
        await Promise.all(
          modes.map(async (mode) => [
            mode,
            await explorers[mode].executeJavaScript(
              `localStorage.getItem('explorer-v2-storage')`,
            ),
          ]),
        ),
      );
      const samePartition = (parent, child) =>
        parent.domain === child.domain &&
        parent.name === child.name &&
        parent.inMemory === child.inMemory &&
        parent.isDefault === child.isDefault;
      if (
        sharedStorage !== "shared-from-app" ||
        storage.persist !== null ||
        storage.memory !== null ||
        storage["inherit+persist"] !== "shared-from-app" ||
        storage["inherit+memory"] !== "shared-from-app" ||
        modes.some(
          (mode) =>
            children[mode].storage.parent !== `${mode}-parent` ||
            children[mode].partition.shared !== true ||
            children[mode].partition.mode !== "shared" ||
            !samePartition(
              states[mode].storagePartition,
              children[mode].partition,
            ),
        ) ||
        !states["inherit+persist"].storagePartition.inherited ||
        !states["inherit+memory"].storagePartition.inherited
      ) {
        throw new Error(
          "Partition inheritance/isolation invariants did not hold",
        );
      }
      explorerStorageResult.textContent = formatValue({
        sharedParentPersistent: {
          storage: sharedStorage,
          partition: sharedState.storagePartition,
        },
        independentPersistent: {
          storage: storage.persist,
          partition: states.persist.storagePartition,
        },
        independentMemory: {
          storage: storage.memory,
          partition: states.memory.storagePartition,
        },
        inheritedPersistent: {
          storage: storage["inherit+persist"],
          partition: states["inherit+persist"].storagePartition,
        },
        inheritedMemory: {
          storage: storage["inherit+memory"],
          partition: states["inherit+memory"].storagePartition,
        },
        omittedPartitionChildren: children,
      });
      logExplorerEvent("api:compare-storage", { sharedStorage, storage });
    } catch (error) {
      explorerStorageResult.textContent = `Storage comparison failed: ${error.message}`;
    }
  });

async function runNestedBubbleDemo(type, stop) {
  const requestId = crypto.randomUUID();
  const configured = new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      nestedBubbleWaiters.delete(requestId);
      reject(new Error("Nested bubble configuration timed out"));
    }, 5000);
    nestedBubbleWaiters.set(requestId, () => {
      clearTimeout(timeout);
      resolve();
    });
  });
  await explorer.postMessage({
    type: "configure-nested-bubble",
    requestId,
    eventType: type,
    stopType: stop ? type : null,
  });
  await configured;
  if (!nestedWindow) throw new Error("Nested Explorer window is unavailable");
  nestedWindow.postMessage(
    { type: "trigger-request", eventType: type },
    { targetOrigin: location.origin },
  );
  setExplorerActionStatus(
    `${stop ? "Stopped" : "Bubbling"} nested ${type} requested`,
    "success",
  );
}
function runNestedBubbleDemoFromControl(type, stop) {
  void runNestedBubbleDemo(type, stop).catch((error) => {
    setExplorerActionStatus(
      `Nested ${type} failed: ${error.message}`,
      "failed",
    );
    logExplorerEvent(`nested:${type}:error`, { message: error.message });
  });
}
document
  .querySelector("#explorer-nested-open-bubble")
  .addEventListener("click", () =>
    runNestedBubbleDemoFromControl("newwindow", false),
  );
document
  .querySelector("#explorer-nested-open-stop")
  .addEventListener("click", () =>
    runNestedBubbleDemoFromControl("newwindow", true),
  );
document
  .querySelector("#explorer-nested-dialog-bubble")
  .addEventListener("click", () =>
    runNestedBubbleDemoFromControl("dialog", false),
  );
document
  .querySelector("#explorer-nested-dialog-stop")
  .addEventListener("click", () =>
    runNestedBubbleDemoFromControl("dialog", true),
  );
document
  .querySelector("#explorer-clear-events")
  .addEventListener("click", () => {
    explorerEvents.replaceChildren();
    setExplorerActionStatus("Event log cleared", "success");
  });

async function checkExplorerV2() {
  try {
    reportExplorerStage("initial-load");
    localStorage.setItem("explorer-v2-storage", "shared-from-app");
    await waitForExplorerLoad(explorer, () => explorerSlot.append(explorer));
    if (checkIndependentPartitionAtStartup) {
      reportExplorerStage("isolated-load");
      const isolatedExplorer = await ensureIsolatedExplorer();
      const isolatedState = await isolatedExplorer.getNavigationState();
      const isolatedProbe = await isolatedExplorer.executeJavaScript(
        `({storage: localStorage.getItem('explorer-v2-storage'), href: location.href})`,
        { expectedDocumentId: isolatedState.documentId },
      );
      const effectivePartition = isolatedState.storagePartition;
      if (
        effectivePartition?.domain !== "tuffite-explorer" ||
        effectivePartition?.name !== "playground-v2-memory" ||
        effectivePartition?.inMemory !== true ||
        effectivePartition?.isDefault !== false ||
        effectivePartition?.requested !== "memory:playground-v2-memory" ||
        effectivePartition?.mode !== "memory" ||
        effectivePartition?.shared !== false
      ) {
        throw new Error(
          `memory partition was not applied: ${JSON.stringify(effectivePartition)}`,
        );
      }
      if (isolatedProbe.storage !== null) {
        throw new Error(
          `memory partition inherited storage at ${isolatedProbe.href}: ` +
            isolatedProbe.storage,
        );
      }
      markExplorerCheck("isolated-partition");
    }
    reportReady("explorer");

    reportExplorerStage("navigation-state");
    const state = await explorer.getNavigationState();
    // Cross-renderer localStorage updates can arrive after the child's first
    // readiness message. Verify the shared partition instead of depending on
    // that one-time notification containing an already-visible value.
    for (let attempt = 0; attempt < 40; attempt++) {
      const storage = await explorer.executeJavaScript(
        `localStorage.getItem('explorer-v2-storage')`,
        { expectedDocumentId: state.documentId },
      );
      if (storage === "shared-from-app") {
        markExplorerCheck("shared-partition");
        break;
      }
      if (attempt === 39) {
        throw new Error("Shared partition storage did not become visible");
      }
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    reportExplorerStage("execute-javascript");
    const scriptResult = await explorer.executeJavaScript(
      `({title: document.title, documentId: ${JSON.stringify(state.documentId)}})`,
      { expectedDocumentId: state.documentId },
    );
    if (
      scriptResult.title !== "Explorer Wire Validation Passed" ||
      scriptResult.documentId !== state.documentId
    ) {
      throw new Error("isolated script result mismatch");
    }
    markExplorerCheck("executeJavaScript");
    reportExplorerStage("parent-message");
    await explorer.postMessage({ type: "parent-probe" });
    reportExplorerStage("nested-message");
    await waitForExplorerCheck("nested-message", 30000);
    reportExplorerStage("navigation-event");
    await explorer.executeJavaScript(
      `document.querySelector('#navigation-link').click(); true`,
      { expectedDocumentId: state.documentId },
    );
    await explorer.executeJavaScript(
      `document.querySelector('#window-open-button').click(); true`,
      { expectedDocumentId: state.documentId },
    );
    await explorer.executeJavaScript(
      `document.querySelector('#popup-link').click(); true`,
      { expectedDocumentId: state.documentId },
    );
    await explorer.executeJavaScript(
      `document.querySelector('#new-window-link').click(); true`,
      { expectedDocumentId: state.documentId },
    );
    if (checkSynchronousDialog) {
      reportExplorerStage("dialog");
      const confirmed = await explorer.executeJavaScript(
        `confirm('Explorer v2 dialog');`,
        { expectedDocumentId: state.documentId },
      );
      if (!confirmed || !dialogResolved) {
        throw new Error("dialog response mismatch");
      }
    }

    reportExplorerStage("history-a");
    await waitForExplorerLoad(explorer, () =>
      explorer.load(new URL("./history-a.html", document.baseURI).href),
    );
    reportExplorerStage("history-b");
    await waitForExplorerLoad(explorer, () =>
      explorer.load(new URL("./history-b.html", document.baseURI).href),
    );
    reportExplorerStage("back");
    const backLoad = waitForExplorerLoad(explorer, () => {});
    if (!(await explorer.back())) throw new Error("back was not accepted");
    await backLoad;
    if (!(await explorer.getNavigationState()).url.includes("history-a.html")) {
      throw new Error("back did not restore history A");
    }
    reportExplorerStage("forward");
    const forwardLoad = waitForExplorerLoad(explorer, () => {});
    if (!(await explorer.forward()))
      throw new Error("forward was not accepted");
    await forwardLoad;
    if (!(await explorer.getNavigationState()).url.includes("history-b.html")) {
      throw new Error("forward did not restore history B");
    }
    markExplorerCheck("history");
    if (openEventsSeen.size !== 2) {
      throw new Error(
        `request events were not emitted independently: ${[...openEventsSeen].sort().join(",")}`,
      );
    }
    reportExplorerStage(`waiting:${[...explorerChecks].sort().join(",")}`);
  } catch (error) {
    reportError("explorer v2", error);
  }
}

function waitForExplorerCheck(name, timeoutMs) {
  return new Promise((resolve, reject) => {
    const started = performance.now();
    const poll = () => {
      if (explorerChecks.has(name)) {
        resolve();
        return;
      }
      if (performance.now() - started >= timeoutMs) {
        reject(new Error(`Explorer check timed out: ${name}`));
        return;
      }
      setTimeout(poll, 50);
    };
    poll();
  });
}
checkExplorerV2();
if (!explorerOnlySmoke) {
  invoke();
  checkNetwork();
  checkPlatformApis();
  checkLargeAsset();
  checkUtility();
  window.playground.commands
    .checkUtilitySession()
    .catch((error) => reportError("utility session", error));
  window.playground.commands
    .checkJavascriptExtension()
    .catch((error) => reportError("JavaScript extension", error));
  checkWindowManagement();
}

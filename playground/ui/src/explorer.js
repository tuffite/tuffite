const mode = new URL(location.href).searchParams.get("mode");
let stopNextNestedType;
let resolveNestedReady;
const nestedReady = new Promise((resolve) => {
  resolveNestedReady = resolve;
});

// Native APIs are granted by committed origin. A malformed business wire
// request must still be rejected, even when this origin has a method grant.
try {
  await window.tuffite.invoke(
    "playground.commands.doSomething",
    new Uint8Array(),
  );
  document.title = "Explorer Wire Validation Failed";
} catch {
  document.title =
    mode === "complete" ? "Explorer v2 Ready" : "Explorer Wire Validation Passed";
  const status = document.querySelector("#role-status");
  if (status) status.textContent = "origin-scoped native APIs · malformed wire rejected";
}

document.querySelector("#window-open-button")?.addEventListener("click", () => {
  window.open("./history-a.html?source=window-open", "_blank");
});
document.querySelector("#popup-link")?.addEventListener("click", () => {
  window.open(
    "./history-a.html?source=popup",
    "_blank",
    "popup,width=480,height=320",
  );
});

const partitionMode =
  {
    isolated: "memory",
    persist: "persist",
    "inherit-persist": "inherit+persist",
    "inherit-memory": "inherit+memory",
  }[mode] ?? "shared";
if (partitionMode !== "shared") {
  localStorage.setItem("explorer-v2-parent-marker", `${partitionMode}-parent`);
  await window.tuffite.postMessage(
    {
      type: `${partitionMode}-ready`,
      storage: localStorage.getItem("explorer-v2-storage"),
    },
    { to: "top" },
  );
} else {
  window.tuffite.addEventListener("message", async (event) => {
    if (event.detail.data?.type === "configure-nested-bubble") {
      const { requestId, stopType } = event.detail.data;
      stopNextNestedType = stopType;
      await nestedReady;
      await window.tuffite.postMessage(
        {
          type: "nested-bubble-configured",
          requestId,
        },
        { to: "top" },
      );
      return;
    }
    if (event.detail.data?.type === "parent-probe") {
      await window.tuffite.postMessage(
        { type: "parent-message-ack", echo: event.detail.data.payload ?? null },
        { to: "top" },
      );
    }
  });
  await window.tuffite.postMessage(
    {
      type: "explorer-ready",
      storage: localStorage.getItem("explorer-v2-storage"),
    },
    { to: "top" },
  );
}

const nested = document.querySelector("#nested");
for (const type of ["navigation", "open", "popup", "newwindow", "dialog"]) {
  nested.addEventListener(type, (event) => {
    const stopped = stopNextNestedType === type;
    if (stopped) {
      stopNextNestedType = undefined;
      event.stopPropagation();
      if (type === "dialog") void event.detail.dismiss();
    }
    void window.tuffite.postMessage(
      {
        type: "nested-bubble-hop",
        eventType: type,
        layer: "level-1-element",
        stopped,
      },
      { to: "top" },
    );
  });
}
const reportNestedStorage = async () => {
  const state = await nested.getNavigationState();
  // Attaching an inner WebContents can expose the initial about:blank
  // loadstop before the requested document commits. Only report the final
  // demo document so the storage proof cannot capture that transient frame.
  if (!state.url.includes("/nested.html")) return;
  nested.removeEventListener("loadstop", reportNestedStorage);
  const storage = await nested.executeJavaScript(
    `({app: localStorage.getItem('explorer-v2-storage'), ` +
      `parent: localStorage.getItem('explorer-v2-parent-marker')})`,
    { expectedDocumentId: state.documentId },
  );
  await window.tuffite.postMessage(
    {
      type: "nested-storage-ready",
      ownerMode: partitionMode,
      partition: state.storagePartition,
      storage,
    },
    { to: "top" },
  );
  resolveNestedReady();
};
nested.addEventListener("loadstop", reportNestedStorage);
nested.src = new URL(
  `./nested.html?owner=${partitionMode}`,
  document.baseURI,
).href;

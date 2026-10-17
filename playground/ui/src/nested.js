const requestId = crypto.randomUUID();
document.querySelector("#nested-open").addEventListener("click", () => {
  void window.tuffite.postMessage(
    { type: "nested-request-triggered", eventType: "open" },
    { targetOrigin: "*" },
  );
  window.open("./history-a.html?source=nested-open", "_blank");
});
document.querySelector("#nested-popup").addEventListener("click", () => {
  window.open(
    "./history-a.html?source=nested-popup",
    "_blank",
    "popup,width=420,height=300",
  );
});
document.querySelector("#nested-dialog").addEventListener("click", () => {
  const accepted = confirm("Dialog emitted by the nested Explorer");
  void window.tuffite.postMessage(
    {
      type: "nested-dialog-result",
      accepted,
    },
    { targetOrigin: "*" },
  );
});

const receive = async (event) => {
  const response = event.data;
  if (response?.type === "trigger-request") {
    const selector =
      response.eventType === "dialog" ? "#nested-dialog" : "#nested-open";
    document.querySelector(selector)?.click();
    return;
  }
  if (
    response?.type !== "notification-result" ||
    response.requestId !== requestId
  ) {
    return;
  }
  document.title = response.shown
    ? "Nested Explorer Ready"
    : "Nested Explorer Message Failed";
  await window.tuffite.postMessage(
    {
      type: "nested-message-complete",
      shown: response.shown === true,
    },
    { targetOrigin: "*" },
  );
};
window.addEventListener("message", receive);
await window.tuffite.postMessage(
  {
    type: "notification-request",
    requestId,
    notification: {
      title: "Nested Explorer",
      body: "Browser-routed postMessage request",
    },
  },
  { targetOrigin: "*" },
);

import "/tuffite.js";
const output = document.querySelector("#event-output");
const events = [];
const requests = [];
const record = (layer, event) => {
  events.push({
    layer,
    type: event.type,
    phase: event.eventPhase,
    bubbles: event.bubbles,
    composed: event.composed,
    cancelable: event.cancelable,
    prevented: event.defaultPrevented,
    detail:
      event.type === "newwindow"
        ? {
            url: event.detail.url,
            disposition: event.detail.disposition,
            from: event.detail.from,
          }
        : event.type === "dialog"
          ? { type: event.detail.type, message: event.detail.message }
          : event.detail,
  });
  output.textContent = JSON.stringify(events.slice(-50), null, 2);
};
const source = document.createElement("explorer");
source.id = "event-source";
source.navigation = "allow";
source.dialogs = "emit";
source.style.cssText = "display:block;width:100%;height:330px";
const handle = (layer, event) => {
  record(layer, event);
  if (document.querySelector("#event-handler").value !== layer) return;
  event.preventDefault();
  const request = event.detail;
  requests.push(request);
  request.signal.addEventListener(
    "abort",
    () =>
      events.push({ type: "request-aborted", layer, requestType: event.type }),
    { once: true },
  );
  if (event.type === "dialog") {
    if (window.eventLab.holdDialog) return;
    void request.accept("event answer").then((result) => {
      window.eventLab.response = result;
    });
  } else {
    const action = document.querySelector("#window-action").value;
    if (action === "hold") return;
    if (action === "discard") {
      void request.discard().then((result) => {
        window.eventLab.response = result;
      });
      return;
    }
    const target = document.createElement("explorer");
    target.navigation = source.navigation;
    target.dialogs = source.dialogs;
    target.id = `event-window-${requests.length}`;
    target.style.cssText = "display:block;width:100%;height:180px";
    document.querySelector("#event-windows").append(target);
    void request
      .attach(target)
      .then((result) => {
        window.eventLab.response = result;
      })
      .catch((error) => {
        window.eventLab.error = error.message;
      });
  }
};
for (const type of ["newwindow", "dialog"]) {
  document.addEventListener(type, (event) => record("capture", event), {
    capture: true,
  });
  source.addEventListener(type, (event) => handle("element", event));
  document.addEventListener(type, (event) => record("bubble", event));
  window.tuffite.addEventListener(type, (event) => handle("top", event));
}
for (const type of [
  "titlechange",
  "loadcommit",
  "loadstart",
  "loadstop",
  "zoomchange",
  "fullscreenchange",
  "pictureinpicturechange",
]) {
  document.addEventListener(type, (event) => record("state", event));
}
window.tuffite.addEventListener("message", (event) => record("message", event));
window.eventLab = { events, requests, source, holdDialog: false };
source.src = new URL("./event-fixture.html", location.href).href;
document.querySelector("#event-owner").append(source);
document.querySelector("#event-open").onclick = () => {
  void source.executeJavaScript(
    "document.querySelector('#open').click(); true",
  );
};
document.querySelector("#event-dialog").onclick = () => {
  void source.executeJavaScript(
    "document.querySelector('#confirm').click(); true",
  );
};
document.querySelector("#event-reset").onclick = () => {
  source.src = new URL(
    "./event-fixture.html?reset=" + Date.now(),
    location.href,
  ).href;
};

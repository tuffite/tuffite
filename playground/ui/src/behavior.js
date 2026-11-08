import "/tuffite.js";
const source = document.createElement("explorer");
source.id = "behavior-source";
source.isolated = true;
source.style.cssText = "display:block;width:100%;height:300px";
source.policy = document.querySelector("#policy").value;
const events = [];
let pending;
const record = (value) => {
  events.push(value);
  document.querySelector("#behavior-output").textContent = JSON.stringify(
    events,
    null,
    2,
  );
};
const clear = () => {
  pending = undefined;
  document.querySelector("#confirmation").hidden = true;
};
for (const type of ["dialog", "externalprotocol", "newwindow", "navigation"]) {
  source.addEventListener(type, (event) => {
    event.preventDefault();
    const request = event.detail;
    record({
      type,
      url: request.url,
      origin: request.initiatingOrigin,
      message: request.message,
    });
    if (type === "navigation") return;
    if (type === "newwindow") {
      void request.discard();
      return;
    }
    pending = request;
    document.querySelector("#request-description").textContent =
      type === "externalprotocol"
        ? `${request.initiatingOrigin} wants to open ${request.url}`
        : request.message;
    document.querySelector("#confirmation").hidden = false;
    request.signal.addEventListener(
      "abort",
      () => {
        record({ type: "canceled", request: type });
        if (pending === request) clear();
      },
      { once: true },
    );
  });
}
document.querySelector("#accept").onclick = async () => {
  if (pending) record({ type: "accepted", result: await pending.accept() });
  clear();
};
document.querySelector("#dismiss").onclick = async () => {
  if (pending) record({ type: "dismissed", result: await pending.dismiss() });
  clear();
};
document.querySelector("#apply").onclick = () => {
  source.policy = document.querySelector("#policy").value;
};
const reload = () =>
  source.load("playground://behavior.fixture/ui/dist/fixtures/behavior.html");
document.querySelector("#reload").onclick = reload;
document.querySelector("#behavior-owner").append(source);
reload();
window.policyLab = {
  source,
  events,
  get pending() {
    return pending;
  },
};

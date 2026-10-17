window.eventProof = crypto.randomUUID();
sessionStorage.setItem("event-proof", window.eventProof);
document.querySelector("#open").onclick = () => {
  window.lastPopup = window.open("./event-popup.html", "_blank");
};
document.querySelector("#confirm").onclick = () => {
  window.dialogResult = confirm("Event model confirm");
};
document.querySelector("#prompt").onclick = () => {
  window.dialogResult = prompt("Event model prompt", "default");
};
document.querySelector("#alert").onclick = () => {
  alert("Event model alert");
  window.dialogResult = "done";
};
window.addEventListener("message", (event) => {
  if (event.data?.type === "event-popup-loaded") window.popupReply = event.data;
});
window.childMode = "bubble";
if (new URL(location.href).searchParams.has("iframe")) {
  const frame = document.createElement("iframe");
  frame.name = "event-frame";
  frame.src = new URL("./event-fixture.html?frame", location.href).href;
  document.querySelector("#child").append(frame);
}
if (new URL(location.href).searchParams.has("nested")) {
  const child = document.createElement("explorer");
  child.id = "nested-child";
  child.navigation = "allow";
  child.dialogs = "emit";
  child.style.cssText = "display:block;width:100%;height:220px";
  for (const type of ["newwindow", "dialog"])
    child.addEventListener(type, (event) => {
      window.lastChildRequest = event.detail;
      if (window.childMode === "stop") event.stopPropagation();
      if (window.childMode === "immediate") event.stopImmediatePropagation();
      if (window.childMode === "claim") {
        event.preventDefault();
        if (type === "newwindow") void event.detail.discard();
        else void event.detail.dismiss();
      }
      void window.tuffite.postMessage(
        { type: "event-hop", eventType: type, mode: window.childMode },
        { targetOrigin: "*" },
      );
    });
  child.src = new URL("./event-fixture.html?leaf", location.href).href;
  document.querySelector("#child").append(child);
}

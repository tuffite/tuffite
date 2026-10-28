document.querySelector("#confirm").onclick = () => {
  document.body.dataset.confirmed = String(
    confirm("Behavior fixture confirmation"),
  );
};
document.querySelector("#popup").onclick = () =>
  window.open("behavior.html?popup", "_blank");

window.startSchemeWorker = () =>
  new Promise((resolve, reject) => {
    const worker = new Worker(new URL("./scheme-worker.js", import.meta.url), {
      type: "module",
    });
    worker.onmessage = (event) => {
      worker.terminate();
      resolve(event.data);
    };
    worker.onerror = (event) => {
      worker.terminate();
      reject(new Error(event.message));
    };
    worker.postMessage("origin");
  });

import "/tuffite.js";
const output = document.querySelector("#result");
const events = [];
window.tuffite?.addEventListener("channel", ({ detail: { channel, data } }) => {
  if (!channel.startsWith("playground.framework.")) return;
  const payload = new TextDecoder().decode(data);
  events.push({ channel, data: payload });
  if (channel === "playground.framework.action" && new URL(payload).pathname === "/dialog/open") {
    const dialog = document.querySelector("#image-dialog");
    if (!dialog.open) dialog.showModal();
  }
  document.querySelector("#events").textContent = JSON.stringify(events, null, 2);
});
async function run() {
  const resource = await (await fetch("/framework/resource.json")).json();
  if (resource.context !== "playground application" || !resource.profile || !resource.window) throw Error("Missing business context or source metadata");
  const file = await (await fetch("/framework/file")).text();
  if (file !== "Resolved from a business-owned file") throw Error("File resource failed");
  const consumed = await fetch("/dialog/open?id=image");
  if (consumed.status !== 204) throw Error("Consumed request did not complete with 204");
  let discarded = false;
  try { await fetch("/framework/discard"); } catch { discarded = true; }
  if (!discarded) throw Error("Discarded fetch unexpectedly succeeded");
  const missing = await fetch("/framework/missing");
  if (missing.status !== 404) throw Error("Unresolved route did not finish with 404");
  const image = document.querySelector("#resource-image");
  await image.decode();
  if (!image.naturalWidth) throw Error("Dynamic image could not be decoded");
  const result = { resource, file, consumed: consumed.status, discarded, missing: missing.status, image: image.naturalWidth };
  output.textContent = JSON.stringify(result, null, 2);
  return result;
}
window.frameworkLab = { run, events };
document.querySelector("#run").addEventListener("click", () => run().catch(error => { output.textContent = error.stack; }));
for (const [id, path] of [["window-shortcut", "window"], ["system-shortcut", "system"]]) {
  document.querySelector(`#${id}`).addEventListener("click", async () => {
    const response = await fetch(`/framework/shortcuts/${path}`);
    document.querySelector("#shortcut-result").textContent = response.ok ? `Registered ${path} shortcut` : await response.text();
  });
}

document.querySelector("#close-dialog").addEventListener("click", () => document.querySelector("#image-dialog").close());

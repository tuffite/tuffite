import "/shell_api.js";
document.querySelector("#origin").textContent = location.origin;
const check = async (call) => {
  try { await call(); return "allowed"; }
  catch (error) { return `denied: ${error.message}`; }
};
window.capabilitySample = {
  run: async () => ({
    origin: location.origin,
    read: await check(() => tuffite.path.directories()),
    write: await check(() => tuffite.storage.set("capability-smoke-forbidden", new Uint8Array([0x6e, 0x6f]))),
    appCommand: await check(() => window.playground.commands.doSomething("origin-capability")),
    createExplorer: typeof document.createElement("explorer").getNavigationState === "function",
  }),
};

const report = await window.capabilitySample.run();
document.body.dataset.capabilityReport = JSON.stringify(report);

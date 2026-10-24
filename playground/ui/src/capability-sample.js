import * as frameworkApi from "/tuffite.js";

import { commands, echo, aux } from "/shell_api.js";
document.querySelector("#origin").textContent = location.origin;
const check = async (call) => {
  try { await call(); return "allowed"; }
  catch (error) { return `denied: ${error.message}`; }
};
const sample = {small:7, large:17n, enabled:true, visible:false, medium:513,
  signedValue:-12345, fraction:1.25, nested:{label:"permission probe", bytes:new Uint8Array([0,1,255])}};
const checkTypedCommand = async (call) => {
  try { await call(); return {allowed:true}; }
  catch (error) { return {allowed:false, status:error.status ?? error.cause?.status, message:error.message}; }
};
window.capabilitySample = {
  run: async () => ({
    origin: location.origin,
    read: await check(() => frameworkApi.path.directories()),
    write: await check(() => frameworkApi.storage.set("capability-smoke-forbidden", new Uint8Array([0x6e, 0x6f]))),
    appCommand: await check(() => commands.doSomething("origin-capability")),
    typedCommand: await checkTypedCommand(() => commands.echoMojom(sample)),
    typedUtility: await checkTypedCommand(() => echo.roundTrip(sample)),
    deniedEndpoint: await checkTypedCommand(() => echo.restricted(sample)),
    deniedService: await checkTypedCommand(() => aux.roundTrip(sample)),
    applicationGlobal: Object.hasOwn(globalThis, "playground"),
    createExplorer: typeof document.createElement("explorer").getNavigationState === "function",
  }),
};

const report = await window.capabilitySample.run();
document.body.dataset.capabilityReport = JSON.stringify(report);

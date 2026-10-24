// Real Browser/Utility process proof, using the packaged generated facade.
import assert from 'node:assert/strict';
// Attach only the main target. Explorer targets can have their own pending
// dialogs; enabling every target through Playwright is unnecessary here.
const port = process.env.TUFFITE_PLAYGROUND_CDP_PORT;
const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
const target = targets.find(item => item.type === 'page' && item.url.includes('/mojom-smoke.html'));
assert.ok(target?.webSocketDebuggerUrl, 'Playground main document exists');
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  const timeout = setTimeout(() => reject(new Error('Opening main CDP target timed out')), 10000);
  socket.addEventListener('open', () => { clearTimeout(timeout); resolve(); }, {once:true});
  socket.addEventListener('error', reject, {once:true});
});
let nextId = 0;
function send(method, params) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      socket.removeEventListener('message', receive);
      reject(new Error(`CDP ${method} timed out`));
    }, 10000);
    function receive(event) {
      const message = JSON.parse(event.data);
      if (message.id !== id) return;
      clearTimeout(timeout);
      socket.removeEventListener('message', receive);
      if (message.error) reject(new Error(JSON.stringify(message.error)));
      else resolve(message.result);
    }
    socket.addEventListener('message', receive);
    socket.send(JSON.stringify({id, method, params}));
  });
}
async function evaluate(fn) {
  const result = await send('Runtime.evaluate', {
    expression:`(${fn.toString()})()`, awaitPromise:true, returnByValue:true,
  });
  if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
  return result.result.value;
}
try {
  const deadline = Date.now() + 40000;
  let state;
  while (Date.now() < deadline) {
    state = await evaluate(() => ({title:document.title, result:window.__mojomSmoke}));
    if (state.result?.browser && state.result?.utility) break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert.deepEqual(state.result, {browser:true, utility:true}, JSON.stringify(state));
  assert.equal(await evaluate(() => Object.hasOwn(globalThis, 'playground')), false, 'No application global');
  assert.equal(await evaluate(() => window.__frameworkModuleOnly), true, 'Framework clients are module-only');
  assert.equal(await evaluate(() => window.__scopedGrantsRejected), true, 'Incompatible scopes fail closed in Browser C++, Browser Rust and Utility Rust');
  assert.deepEqual(await evaluate(() => window.__typedUtilitySecurity), {
    rawBypassDenied:true, rawCancelDenied:true, forgedServiceDenied:true, schemaMismatchRejected:true,
  });
  const ownership = await evaluate(() => window.__interfaceOwnership);
  assert.equal(ownership.sharedState, true);
  assert.equal(ownership.separateProcess, true);
  assert.notEqual(ownership.browserPid, ownership.utilityPid);
  const boundary = await evaluate(() => window.__mojomBoundary());
  assert.deepEqual(boundary, {large:'18446744073709551615',signed:-2147483648,
    label:'boundary 🌍',bytes:0,enabled:false,visible:true,rejected:true});
  if (process.env.TUFFITE_MOJOM_BENCHMARK === '1') {
    const timings = await evaluate(async () => {
      const {commands, diagnostics, echo, aux} = await import(new URL('./shell_api.js', document.baseURI).href);
      const framework = await import('/tuffite.js');
      const report = {};
      const small = {small:7,large:17n,enabled:true,visible:false,medium:513,
        signedValue:-12345,fraction:1.25,nested:{label:'benchmark',bytes:new Uint8Array(32)}};
      const large = {...small,nested:{label:'large',bytes:new Uint8Array(1024*1024).fill(173)}};
      // Utility unary replies currently have a 1 MiB total frame budget.
      const utilityLarge = {...small,nested:{label:'large',bytes:new Uint8Array(512*1024).fill(173)}};
      const fixture = `mojom-performance-${Date.now()}.bin`;
      await framework.fs.writeFile(framework.BaseDirectory.Data, fixture, large.nested.bytes);
      const probes = {
        browserCpp: () => framework.path.directories(),
        browserCpp1MiB: async () => { const r=await framework.fs.readFile(framework.BaseDirectory.Data,fixture); if(r.length!==1048576||r[1048575]!==173)throw new Error('bad C++ bytes'); },
        browserRust: () => diagnostics.processId(),
        utilityRust: () => aux.processId(),
        browserRustSmall: async () => { const r=await commands.echoMojom(small); if(r.large!==17n)throw new Error('bad Browser reply'); },
        utilityRustSmall: async () => { const r=await echo.roundTrip(small); if(r.large!==17n)throw new Error('bad Utility reply'); },
        browserRust1MiB: async () => { const r=await commands.echoMojom(large); if(r.nested.bytes.length!==1048576||r.nested.bytes[1048575]!==173)throw new Error('bad Browser bytes'); },
        utilityRust512KiB: async () => { const r=await echo.roundTrip(utilityLarge); if(r.nested.bytes.length!==524288||r.nested.bytes[524287]!==173)throw new Error('bad Utility bytes'); },
      };
      try { for (const [name,probe] of Object.entries(probes)) {
        const count = name.includes('1MiB') || name.includes('512KiB') ? 12 : 100;
        for(let i=0;i<10;i++)await probe();
        const samples=[];
        for(let i=0;i<count;i++){const start=performance.now();await probe();samples.push(performance.now()-start);}
        samples.sort((a,b)=>a-b);
        report[name]={count,p50Ms:samples[Math.floor(count*.5)],p95Ms:samples[Math.floor(count*.95)]};
      } } finally { await framework.fs.removeFile(framework.BaseDirectory.Data,fixture); }
      const start=performance.now();
      await Promise.all(Array.from({length:64},()=>probes.utilityRustSmall()));
      report.utilityConcurrent64Ms=performance.now()-start;
      return report;
    });
    console.log(JSON.stringify({performance:timings}));
  }
  console.log(JSON.stringify({mojom_browser_rust:true,mojom_utility_rust:true,typed_boundaries:true,typed_utility_routing:true,shared_worker_state:true,separate_process:true,raw_bypass_denied:true,forged_route_denied:true}));
} finally { socket.close(); }

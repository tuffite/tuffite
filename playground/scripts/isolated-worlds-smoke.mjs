// Packaged DOM isolated-world component proof.
import assert from 'node:assert/strict';
// Attach only the main target. Explorer targets can have their own pending
// dialogs; enabling every target through Playwright is unnecessary here.
const port = process.env.TUFFITE_PLAYGROUND_CDP_PORT;
let target;
const startupDeadline = Date.now() + 40000;
while (Date.now() < startupDeadline) {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  target = targets.find(item => item.type === 'page' && item.url.startsWith('playground://main/'));
  if (target?.webSocketDebuggerUrl) break;
  await new Promise(resolve => setTimeout(resolve, 100));
}
assert.ok(target?.webSocketDebuggerUrl, 'Playground main document exists after Browser initialization');
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
  await send('Page.enable', {});
  await send('Page.navigate', {url: 'playground://main/ui/dist/isolated.html'});
  let state;
  const deadline = Date.now() + 40000;
  while (Date.now() < deadline) {
    state = await evaluate(() => globalThis.__isolatedSmoke);
    if (state) break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert.equal(state?.error, undefined, JSON.stringify(state));
  for (const name of ['mounted','globalsIsolated','registryIsolated','updated','invalidPropertiesRejected','unregisteredDenied','removedDenied','nativeRegistrationOnly'])
    assert.equal(state?.[name], true, name);
  const frameTree = await send('Page.getFrameTree', {});
  const ordinary = await send('Page.createIsolatedWorld', {
    frameId: frameTree.frameTree.frame.id, worldName: 'ordinary-inspector-world',
  });
  const ordinaryResult = await send('Runtime.evaluate', {
    contextId: ordinary.executionContextId, returnByValue: true,
    expression: `(() => {
      let denied = false;
      try { new (class extends HTMLElement {})(); } catch { denied = true; }
      return {denied, registry: window.customElements};
    })()`,
  });
  assert.deepEqual(ordinaryResult.result.value, {denied:true,registry:null});
  assert.equal(await evaluate(() => {
    class MainCard extends HTMLElement {}
    customElements.define('weather-card', MainCard);
    return document.createElement('weather-card') instanceof MainCard;
  }), true, 'main-world standard components retain their own registry');
  // CDP pierces closed shadow roots only for inspection. No host DOM backdoor
  // is added to the public API for testing.
  const doc = await send('DOM.getDocument', {depth: -1, pierce: true});
  function collect(node, name, result = []) {
    if (node.nodeName?.toLowerCase() === name) result.push(node);
    for (const child of [...(node.children ?? []), ...(node.shadowRoots ?? [])]) collect(child, name, result);
    return result;
  }
  const cards = collect(doc.root, 'weather-card');
  assert.equal(cards.length, 2);
  const attrs = node => Object.fromEntries(Array.from({length:node.attributes.length/2}, (_,i) => node.attributes.slice(i*2,i*2+2)));
  for (const card of cards) {
    assert.equal(attrs(card)['data-registry-local'], 'true');
    assert.equal(attrs(card)['data-foreign-denied'], 'true');
    assert.equal(attrs(card)['data-main-registry'], 'null');
    assert.equal(attrs(card)['data-shell-api'], 'undefined');
    // Host mutation invokes the extension callback in its owning world.
    await send('DOM.setAttributeValue', {nodeId:card.nodeId, name:'data-probe', value:'host'});
  }
  const buttons = collect(doc.root, 'button').filter(node => node.attributes.length === 0);
  assert.equal(buttons.length, 2);
  for (const button of buttons) {
    const object = await send('DOM.resolveNode', {nodeId:button.nodeId});
    await send('Runtime.callFunctionOn', {objectId:object.object.objectId, functionDeclaration:'function(){this.click()}', returnByValue:true});
  }
  await new Promise(resolve => setTimeout(resolve, 100));
  const after = await send('DOM.getDocument', {depth:-1,pierce:true});
  const updated = collect(after.root,'weather-card').map(attrs);
  assert.deepEqual(updated.map(item => item['data-world']).sort(), ['weather','weatherOther']);
  for (const item of updated) {
    assert.equal(item['data-callback-world'], item['data-world']);
    assert.equal(item['data-main-secret'], 'undefined');
    assert.ok(item['data-reply'], JSON.stringify(item));
    assert.equal(item['data-error'], undefined);
  }
  assert.equal(collect(after.root,'world-detail').length, 2, 'nested scoped components upgrade');
  assert.deepEqual(updated.map(item=>JSON.parse(item['data-reply']).value).sort((a,b)=>a-b), [20,70]);
  await evaluate(async () => {
    await globalThis.__isolatedLab.views[1].update({probe:'cancel'});
  });
  await new Promise(resolve => setTimeout(resolve, 100));
  assert.deepEqual(await evaluate(() => globalThis.__isolatedWaits), {started:1,aborted:1});
  await evaluate(async () => {
    await globalThis.__isolatedLab.views[0].update({probe:'pending'});
  });
  await new Promise(resolve => setTimeout(resolve, 50));
  const disposal = await evaluate(async () => {
    const {first,second,views} = globalThis.__isolatedLab;
    await views[0].dispose(); await views[0].dispose();
    let denied = false;
    try { await views[0].update({data:{value:9}}); } catch(error) { denied = error.name === 'InvalidStateError'; }
    await first.dispose(); await first.dispose(); await second.dispose();
    let mountDenied = false;
    try { await first.mount(document.body,{tag:'weather-card'}); } catch(error) { mountDenied = error.name === 'InvalidStateError'; }
    return {denied,mountDenied,remaining:document.querySelectorAll('[data-isolated-world]').length};
  });
  assert.deepEqual(disposal,{denied:true,mountDenied:true,remaining:0});
  assert.deepEqual(await evaluate(() => globalThis.__isolatedWaits), {started:2,aborted:2});
  await send('Page.navigate', {url:'playground://public/ui/dist/isolated.html'});
  let deniedState;
  const deniedDeadline = Date.now() + 10000;
  while (Date.now() < deniedDeadline) {
    deniedState = await evaluate(() => globalThis.__isolatedSmoke);
    if (deniedState?.error) break;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.match(deniedState?.error ?? '', /unavailable or denied/);
  console.log(JSON.stringify({isolated_worlds:true,dynamic_registration:true,removed_registration_denied:true,scoped_components:true,property_setters:true,callbacks:true,request_reply:true,disposal:true}));
} finally { socket.close(); }

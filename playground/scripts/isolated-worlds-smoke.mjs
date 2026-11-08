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
const contexts = new Map();
socket.addEventListener('message', event => {
  const message = JSON.parse(event.data);
  if (message.method === 'Runtime.executionContextCreated') {
    const context = message.params.context;
    contexts.set(context.id, context);
  } else if (message.method === 'Runtime.executionContextDestroyed') {
    contexts.delete(message.params.executionContextId);
  } else if (message.method === 'Runtime.executionContextsCleared') {
    contexts.clear();
  }
});
let nextId = 0;
function send(method, params, connection = socket) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      connection.removeEventListener('message', receive);
      reject(new Error(`CDP ${method} timed out`));
    }, 10000);
    function receive(event) {
      const message = JSON.parse(event.data);
      if (message.id !== id) return;
      clearTimeout(timeout);
      connection.removeEventListener('message', receive);
      if (message.error) reject(new Error(JSON.stringify(message.error)));
      else resolve(message.result);
    }
    connection.addEventListener('message', receive);
    connection.send(JSON.stringify({id, method, params}));
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
  await send('Runtime.enable', {});
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
  for (const name of ['mounted','globalsIsolated','registryIsolated','updated','invalidPropertiesRejected','unregisteredDenied','removedDenied','nativeRegistrationOnly','dynamicPackage','failedModulesReleaseWorldsAndLeases'])
    assert.equal(state?.[name], true, name);
  for (const name of ['weather', 'weatherOther']) {
    const context = [...contexts.values()].find(context => context.name === name);
    assert.ok(context, `Registered isolated world: ${name}`);
    const result = await send('Runtime.evaluate', {
      contextId: context.id, returnByValue: true,
      expression: `(() => {
        if (typeof globalThis.Mojo?.bindInterface !== 'function') return true;
        const {handle0, handle1} = Mojo.createMessagePipe();
        try {
          Mojo.bindInterface('playground.wire.Browser', handle1);
          return false;
        } catch (error) {
          return error.name === 'NotAllowedError';
        } finally {
          handle0.close();
          handle1.close();
        }
      })()`,
    });
    assert.equal(result.result.value, true, `${name} cannot bind native interfaces`);
  }
  const frameTree = await send('Page.getFrameTree', {});
  const ordinary = await send('Page.createIsolatedWorld', {
    frameId: frameTree.frameTree.frame.id, worldName: 'ordinary-inspector-world',
  });
  const ordinaryResult = await send('Runtime.evaluate', {
    contextId: ordinary.executionContextId, returnByValue: true,
    expression: `(() => {
      let denied = false;
      try { new (class extends HTMLElement {})(); } catch { denied = true; }
      let mojoDenied = typeof globalThis.Mojo?.bindInterface !== 'function';
      if (!mojoDenied) {
        const {handle0, handle1} = Mojo.createMessagePipe();
        try {
          Mojo.bindInterface('playground.wire.Browser', handle1);
        } catch (error) {
          mojoDenied = error.name === 'NotAllowedError';
        } finally {
          handle0.close();
          handle1.close();
        }
      }
      return {denied, registry: window.customElements, mojoDenied};
    })()`,
  });
  assert.deepEqual(ordinaryResult.result.value, {denied:true,registry:null,mojoDenied:true});
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
    const modules = JSON.parse(attrs(card)['data-modules']);
    assert.equal(modules.label, 'native ESM');
    assert.equal(modules.lazy, 'dynamic import');
    assert.equal(modules.city, 'Shanghai');
    assert.equal(modules.escapeDenied, true);
    assert.equal(modules.networkDenied, true);
    assert.match(modules.moduleUrl, /\/.tuffite-isolated\/[^/]+\/details.js$/);
    const object = await send('DOM.resolveNode', {nodeId: card.nodeId});
    const computed = await send('Runtime.callFunctionOn', {
      objectId: object.object.objectId,
      functionDeclaration: `function() {
        const style = getComputedStyle(this);
        return {padding: style.padding, border: style.borderTopColor, background: style.backgroundImage};
      }`,
      returnByValue: true,
    });
    assert.deepEqual({...computed.result.value, background: undefined}, {
      padding: '16px', border: 'rgb(136, 136, 136)', background: undefined,
    }, 'native CSS module applies inside the component shadow root');
    assert.match(computed.result.value.background, /^url\("playground:\/\/main\/\.tuffite-isolated\//, 'CSS relative url() resolves in the package');
    // Host mutation invokes the extension callback in its owning world.
    await send('DOM.setAttributeValue', {nodeId:card.nodeId, name:'data-probe', value:'host'});
  }
  const images = collect(doc.root, 'img');
  assert.equal(images.length, 2);
  for (const image of images) {
    const object = await send('DOM.resolveNode', {nodeId: image.nodeId});
    const decoded = await send('Runtime.callFunctionOn', {
      objectId: object.object.objectId,
      functionDeclaration: 'async function() { await this.decode(); return {url: this.currentSrc, width: this.naturalWidth}; }',
      awaitPromise: true, returnByValue: true,
    });
    assert.equal(decoded.exceptionDetails, undefined, 'bundled image decodes under native CSP');
    assert.match(decoded.result.value.url, /^playground:\/\/main\/\.tuffite-isolated\//);
    assert.equal(decoded.result.value.width, 24);
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
  // Application routes also run in Explorer WebContents (for example thread
  // pages). Exercise the real browser-owned role and committed-origin checks.
  for (const origin of ['main', 'public']) {
    const url = `playground://${origin}/ui/dist/isolated.html?explorer-isolated-smoke`;
    await evaluate(() => {
      const explorer = document.createElement('explorer');
      explorer.id = 'isolated-explorer-smoke';
      explorer.behavior = "navigation: browser;";
      explorer.style.cssText = 'display:block;width:500px;height:300px';
      document.body.append(explorer);
    });
    await send('Runtime.evaluate', {expression: `document.querySelector('#isolated-explorer-smoke').src = ${JSON.stringify(url)}`});
    let guest;
    const guestDeadline = Date.now() + 15000;
    while (Date.now() < guestDeadline) {
      const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      guest = targets.find(item => item.type === 'webview' && item.url === url);
      if (guest?.webSocketDebuggerUrl) break;
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    assert.ok(guest?.webSocketDebuggerUrl, `Explorer ${origin} target`);
    const guestSocket = new WebSocket(guest.webSocketDebuggerUrl);
    try {
      await new Promise((resolve, reject) => {
        guestSocket.addEventListener('open', resolve, {once:true});
        guestSocket.addEventListener('error', reject, {once:true});
      });
      let guestState;
      const stateDeadline = Date.now() + 15000;
      while (Date.now() < stateDeadline) {
        const result = await send('Runtime.evaluate', {
          expression:'globalThis.__isolatedSmoke', returnByValue:true,
        }, guestSocket);
        guestState = result.result.value;
        if (guestState) break;
        await new Promise(resolve => setTimeout(resolve, 50));
      }
      if (origin === 'main') {
        assert.equal(guestState?.error, undefined, JSON.stringify(guestState));
        assert.equal(guestState?.mounted, true, 'authorized Explorer can mount isolated UI');
      } else {
        assert.match(guestState?.error ?? '', /unavailable or denied/, 'other Explorer origins remain denied');
      }
    } finally {
      guestSocket.close();
      await evaluate(() => document.querySelector('#isolated-explorer-smoke').remove());
    }
  }
  await send('Page.navigate', {url:'playground://public/ui/dist/isolated.html'});
  let deniedState;
  const deniedDeadline = Date.now() + 10000;
  while (Date.now() < deniedDeadline) {
    deniedState = await evaluate(() => globalThis.__isolatedSmoke);
    if (deniedState?.error) break;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.match(deniedState?.error ?? '', /unavailable or denied/);
  console.log(JSON.stringify({isolated_worlds:true,native_esm:true,css_modules:true,json_modules:true,relative_assets:true,failed_entry_cleanup:true,dynamic_registration:true,removed_registration_denied:true,scoped_components:true,property_setters:true,callbacks:true,request_reply:true,disposal:true,explorer_authorized:true,explorer_origin_denied:true}));
} finally { socket.close(); }

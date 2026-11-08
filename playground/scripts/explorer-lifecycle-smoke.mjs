// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
import assert from 'node:assert/strict';
import process from 'node:process';
import { createServer } from 'node:http';

// Attach only to the app. Auto-attaching to child targets would correctly make
// them ineligible for suspension because they are being debugged.
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257);
const server = createServer((_request, response) => {
  response.setHeader('Content-Type', 'text/html');
  response.end('<!doctype html><title>Explorer lifecycle fixture</title><input><p>Idle page</p>');
});
await new Promise((resolve) =>
  server.listen(Number(process.env.TUFFITE_EXPLORER_RELOAD_HTTP_PORT || 0), '::', resolve),
);
const origin = `http://127.0.0.1:${server.address().port}`;
const other = `http://localhost:${server.address().port}`;
const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
const app = targets.find((target) => /\/(index\.html|main)$/.test(new URL(target.url).pathname));
assert.ok(app, 'Missing application target');
const socket = new WebSocket(app.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener('open', resolve, { once: true });
  socket.addEventListener('error', reject, { once: true });
});
let nextId = 0;
const pending = new Map();
socket.addEventListener('message', (event) => {
  const reply = JSON.parse(event.data);
  const request = pending.get(reply.id);
  if (!request) return;
  pending.delete(reply.id);
  clearTimeout(request.timer);
  if (reply.error || reply.result?.exceptionDetails) request.reject(Error(JSON.stringify(reply)));
  else request.resolve(reply.result.result.value);
});
function evaluate(expression) {
  return new Promise((resolve, reject) => {
    const id = ++nextId;
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(Error(`Evaluation timed out: ${expression}`));
    }, 15000);
    pending.set(id, { resolve, reject, timer });
    socket.send(
      JSON.stringify({
        id,
        method: 'Runtime.evaluate',
        params: {
          expression,
          awaitPromise: true,
          returnByValue: true,
        },
      }),
    );
  });
}
async function until(expression, description, timeout = 15000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const value = await evaluate(expression);
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw Error(description);
}
const state = (id) => evaluate(`lifecycleViews[${JSON.stringify(id)}].getNavigationState()`);
try {
  await until("document.readyState !== 'loading' && !!document.body",
    'Application document failed to load');
  await evaluate(`(() => {
    window.lifecycleViews = {};
    const fixture = document.createElement('div');
    fixture.id = 'lifecycle-fixture';
    fixture.style.cssText = 'position:fixed;inset:0;background:white;z-index:99999';
    document.body.append(fixture);
    for (const [id, options] of Object.entries({
      a: {}, b: {policy:'discard: never;'},
      keepAlive: {policy:'freeze: never; discard: never;'},
      partition: {partition:'memory:lifecycle'}, cross: {cross:true}
    })) {
      const element = document.createElement('explorer');
      element.style.cssText = 'display:block;width:300px;height:100px';
      element.policy = 'navigation: browser;';
      for (const [key,value] of Object.entries(options)) if(key !== 'cross') key === 'policy' ? element.policy += value : element[key] = value;
      element.src = (options.cross ? ${JSON.stringify(other)} : ${JSON.stringify(origin)}) + '/'+id;
      lifecycleViews[id] = element;
      fixture.append(element);
    }
  })()`);
  await until(
    `(async () => (await Promise.all(Object.values(lifecycleViews).map(e=>e.getNavigationState()))).every(s=>s.documentId && !s.isLoading && s.url.startsWith('http')))()`,
    'Fixtures failed to load',
  );
  assert.equal(await evaluate(`(() => {const e=lifecycleViews.a; return typeof e.policy==='string' && !('behavior' in e) && !('backgroundPolicy' in e) && !('contextMenuEnabled' in e);})()`), true, 'Policy is the only public settings entry');
  const initial = Object.fromEntries(
    await Promise.all(
      ['a', 'b', 'keepAlive', 'partition', 'cross'].map(async (id) => [id, await state(id)]),
    ),
  );
  assert.equal(
    initial.a.rendererProcess.id,
    initial.b.rendererProcess.id,
    'Same-site pages should share',
  );
  for (const id of ['partition', 'cross']) {
    assert.notEqual(
      initial.a.rendererProcess.id,
      initial[id].rendererProcess.id,
      `${id} must remain isolated`,
    );
  }
  assert.equal(
    initial.a.rendererProcess.id,
    initial.keepAlive.rendererProcess.id,
    'Background policy must not select a separate process',
  );
  assert.notEqual(initial.a.rendererProcess.id, initial.a.rendererProcess.ownerId);
  await evaluate(`lifecycleViews.a.src=${JSON.stringify(other + '/navigated')}`);
  await until(
    `(async()=>{const s=await lifecycleViews.a.getNavigationState();return !s.isLoading && s.url===${JSON.stringify(other + '/navigated')}})()`,
    'Cross-site navigation failed',
  );
  assert.equal((await state('a')).rendererProcess.id, initial.cross.rendererProcess.id);
  await evaluate('lifecycleViews.a.back()');
  await until(
    `(async()=>{const s=await lifecycleViews.a.getNavigationState();return !s.isLoading && s.url===${JSON.stringify(origin + '/a')}})()`,
    'Back navigation failed',
  );
  assert.equal((await state('a')).rendererProcess.id, initial.b.rendererProcess.id);
  await evaluate(
    "lifecycleViews.a.style.display='none'; lifecycleViews.keepAlive.style.display='none'",
  );
  await until(
    '(async()=> (await lifecycleViews.a.getNavigationState()).frozen)()',
    'Hidden page did not freeze',
    45000,
  );
  assert.equal((await state('b')).frozen, false, 'Visible sibling must keep running');
  assert.equal((await state('keepAlive')).frozen, false, 'Keep-alive must prevent freezing');
  assert.equal(await evaluate('lifecycleViews.keepAlive.discard()'), true, 'Explicit discard remains independent of automatic policy');
  const before = await state('a');
  assert.equal(await evaluate('lifecycleViews.a.discard()'), true);
  await until(
    '(async()=> (await lifecycleViews.a.getNavigationState()).discarded)()',
    'Discard not observed',
  );
  assert.equal(
    (await state('b')).documentId,
    initial.b.documentId,
    'Discard must preserve sibling document',
  );
  await evaluate("lifecycleViews.a.style.display='block'");
  await until(
    `(async()=>{const s=await lifecycleViews.a.getNavigationState();return !s.isLoading && !s.discarded && s.documentId!==${JSON.stringify(before.documentId)}})()`,
    'Discarded page failed to restore',
  ).catch(async (error) => {
    const current = await state('a');
    const layout = await evaluate(`(() => { const e=lifecycleViews.a; const r=e.getBoundingClientRect(); return {display:getComputedStyle(e).display,rect:[r.x,r.y,r.width,r.height],visibility:document.visibilityState,hasContentWindow:!!e.contentWindow,outerHTML:e.outerHTML}; })()`);
    throw Error(`${error.message}: ${JSON.stringify({before,current,layout})}`);
  });
  assert.equal((await state('a')).url, before.url);
  assert.equal(
    await evaluate('lifecycleViews.a.discard()'),
    false,
    'Visible page must reject discard',
  );
  // Unlike the shared-sibling case above, discarding this partition's only
  // page lets its renderer exit and destroys the child view. Owner visibility
  // must still reach the discarded WebContents to initiate restoration.
  await evaluate(`(() => {
    const e = document.createElement('explorer');
    e.partition = 'memory:last-discard';
    e.policy = 'navigation: browser; freeze: never; discard: never';
    e.style.cssText = 'display:block;width:300px;height:100px';
    e.src = ${JSON.stringify(origin + '/solo')};
    lifecycleViews.solo = e;
    document.querySelector('#lifecycle-fixture').append(e);
  })()`);
  await until(`(async()=>{const s=await lifecycleViews.solo.getNavigationState();return !s.isLoading && s.url===${JSON.stringify(origin + '/solo')} && !!s.documentId})()`,
    'Single-process fixture failed to load');
  const solo = await state('solo');
  await evaluate("lifecycleViews.solo.style.display='none'");
  await until('lifecycleViews.solo.discard()', 'Single-process fixture did not discard');
  await new Promise(resolve => setTimeout(resolve, 3000));
  assert.equal((await state('solo')).discarded, true);
  await evaluate("lifecycleViews.solo.style.display='block'");
  await until(`(async()=>{const s=await lifecycleViews.solo.getNavigationState();return !s.isLoading && !s.discarded && s.documentId!==${JSON.stringify(solo.documentId)}})()`,
    'The last discarded page failed to restore after its renderer exited');
  console.log(
    'PASS Explorer same-site reuse, isolation boundaries, navigation, freeze, keep-alive, shared-page and last-renderer discard/restoration',
  );
} finally {
  await evaluate(
    "document.querySelector('#lifecycle-fixture')?.remove(); delete window.lifecycleViews",
  ).catch(() => {});
  socket.close();
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
}

import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
const require = createRequire(new URL('../ui/package.json', import.meta.url));
const {chromium} = require('playwright-core');
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257);
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
for (const context of browser.contexts()) {
  const preserve = page => page.on('dialog', () => {});
  context.pages().forEach(preserve);
  context.on('page', preserve);
}
const pages = () => browser.contexts().flatMap(context => context.pages());
const app = pages().find(page => page.url().includes('/index.html'));
assert.ok(app);
const base = new URL('./', app.url()).href;
const wait = async (check, message) => {
  const end = Date.now() + 20000;
  do {
    const result = await Promise.race([check(), new Promise((_,reject) => setTimeout(() => reject(new Error(`Blocked check: ${message}`)), 8000))]);
    if (result) return;
    await new Promise(resolve => setTimeout(resolve, 50));
  } while (Date.now() < end);
  throw new Error(message);
};
// Read our fixture's main-world globals with the test debugger. The public
// Explorer API deliberately does not grant arbitrary main-world execution.
const rawEvaluate = async (url, expression) => {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  const target = targets.find(item => item.url === url);
  assert.ok(target, `Missing fixture target: ${url}`);
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve,reject) => { socket.addEventListener('open',resolve,{once:true}); socket.addEventListener('error',reject,{once:true}); });
  try {
    return await new Promise((resolve,reject) => {
      const timer = setTimeout(() => reject(new Error('Fixture Runtime.evaluate timed out')), 8000);
      socket.addEventListener('message', event => {
        const reply = JSON.parse(event.data); if (reply.id !== 1) return;
        clearTimeout(timer);
        if (reply.error || reply.result?.exceptionDetails) reject(new Error(JSON.stringify(reply.error || reply.result.exceptionDetails)));
        else resolve(reply.result.result.value);
      });
      socket.send(JSON.stringify({id:1, method:'Runtime.evaluate', params:{expression,returnByValue:true,userGesture:true}}));
    });
  } finally { socket.close(); }
};
const execute = async expression => rawEvaluate((await state()).url, expression);
const select = (id,value) => app.evaluate(({id,value}) => { document.querySelector(id).value = value; }, {id,value});
const state = () => app.locator('#event-source').evaluate(element => element.getNavigationState());
const count = (layer, type = 'newwindow') => app.evaluate(({layer,type}) => window.eventLab.events.filter(event => event.layer === layer && event.type === type).length, {layer,type});
const ready = async suffix => {
  await wait(async () => { const value = await state(); return value.url.includes(suffix) && !value.isLoading; }, 'Source did not load');
};
const reset = async (suffix = '') => {
  console.log(`reset=${suffix}:start`);
  await app.locator('#event-source').evaluate((element,url) => { element.src = url; }, new URL(`event-fixture.html${suffix}`, base).href);
  await ready(`event-fixture.html${suffix}`);
  console.log(`reset=${suffix}:ready`);
};
const open = async () => {
  const started = Date.now();
  const invocation = app.locator('#event-source').evaluate(element => element.executeJavaScript("document.body.dataset.started = String(Number(document.body.dataset.started || 0) + 1); document.querySelector('#open').click(); document.body.dataset.finished = document.body.dataset.started; true"));
  // Use the same deadline as other native lifecycle checks and report slow
  // completions separately so timing regressions remain visible.
  const completed = await Promise.race([invocation.then(() => true), new Promise(resolve => setTimeout(() => resolve(false), 20000))]);
  if (completed && Date.now() - started > 3000) console.log(`source-open-callback=slow (${Date.now() - started}ms)`);
  if (!completed) {
    console.log('source-open-callback=waiting');
    console.log(await execute('({started:document.body.dataset.started,finished:document.body.dataset.finished,closed:window.lastPopup?.closed})'));
    throw new Error('Source window.open did not finish');
  }
};
let postBody;
const server = createServer((request,response) => {
  if (request.url === '/fixture') {
    response.setHeader('Content-Type','text/html');
    response.end(`<!doctype html><title>Same-origin window fixture</title><button id="open">Open</button><script>
      window.eventProof = crypto.randomUUID(); sessionStorage.setItem('event-proof', window.eventProof);
      document.querySelector('#open').onclick = () => { window.lastPopup = window.open('/popup', '_blank'); };
      window.addEventListener('message', event => { if (event.data?.type === 'event-popup-loaded') window.popupReply = event.data; });
    </script>`);
    return;
  }
  if (request.url === '/popup') {
    response.setHeader('Content-Type','text/html');
    response.end(`<!doctype html><title>Same-origin original popup</title><script>
      window.popupProof = {opener: !!window.opener, storage: sessionStorage.getItem('event-proof'), marker: window.opener?.eventProof};
      window.opener?.postMessage({type:'event-popup-loaded', ...window.popupProof}, '*');
    </script>`);
    return;
  }
  let body = ''; request.on('data', chunk => { body += chunk; });
  request.on('end', () => {
    postBody = {method:request.method, body};
    response.setHeader('Content-Type','text/html');
    response.end('<!doctype html><title>Original POST window</title><p>POST preserved</p>');
  });
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
try {
  await app.goto(new URL('events.html',base).href);
  await ready('event-fixture.html');
  assert.equal((await state()).fullscreen, false);
  assert.equal((await state()).pictureInPicture, 'none');
  await app.locator('#event-source').evaluate((element,url) => { element.src = url; }, `http://127.0.0.1:${server.address().port}/fixture`);
  await ready('/fixture');
  await open();
  await wait(() => app.evaluate(() => window.eventLab.response === true || !!window.eventLab.error), 'Original window attach did not finish');
  assert.equal(await app.evaluate(() => window.eventLab.error), undefined);
  await wait(() => execute('!!window.popupReply'), 'Original opener did not receive popup message');
  const proof = await execute('({proof:window.eventProof, reply:window.popupReply, closed:window.lastPopup.closed})');
  assert.equal(proof.reply.opener,true); assert.equal(proof.reply.storage,proof.proof); assert.equal(proof.reply.marker,proof.proof); assert.equal(proof.closed,false);
  assert.equal(await app.evaluate(() => window.eventLab.requests[0].discard()), false);
  assert.equal(await app.evaluate(() => window.eventLab.requests[0].signal.aborted), false);
  assert.deepEqual(await app.evaluate(() => window.eventLab.events.filter(event => event.type === 'newwindow').map(event => event.layer)), ['capture','element','bubble','top']);
  assert.ok(await app.evaluate(() => window.eventLab.events.filter(event => event.type === 'newwindow').every(event => event.bubbles && event.composed && event.cancelable)));
  console.log('original-window=passed (opener, WindowProxy, sessionStorage, capture/bubble/top, one-shot)');
  await reset('?controls');

  // A local propagation stop does not claim the browser request.
  await select('#window-action','discard');
  const beforeStop = await count('top');
  console.log('local-stop=start');
  await app.locator('#event-source').evaluate(element => element.addEventListener('newwindow', event => event.stopPropagation(), {once:true}));
  await open();
  console.log('local-stop=opened');
  await wait(async () => await count('top') === beforeStop + 1, 'stopPropagation incorrectly stopped the outer fallback');
  console.log('local-stop=checked');
  const beforeClaim = await count('top');
  await select('#event-handler','element');
  console.log('element-claim=start');
  await open();
  console.log('element-claim=opened');
  await wait(async () => await count('element') > beforeClaim, 'Element did not receive request');
  await app.waitForTimeout(250);
  assert.equal(await count('top'), beforeClaim);
  console.log('claim=passed (preventDefault claims, stopPropagation remains local)');

  await select('#window-action','attach');
  await app.evaluate(() => { window.eventLab.response = undefined; });
  await open();
  await wait(() => app.evaluate(() => window.eventLab.response !== undefined || !!window.eventLab.error), 'Element-side original-window attach did not finish');
  assert.equal(await app.evaluate(()=>window.eventLab.response), true);
  assert.equal(await app.evaluate(() => window.eventLab.requests.at(-1).discard()), false);
  assert.equal(await app.evaluate(() => window.eventLab.requests.at(-1).signal.aborted), false);
  console.log('element-window=passed (native controller attach and one-shot)');

  // Canceling a source invalidates retained controllers and aborts their signal.
  await select('#window-action','hold');
  const heldCount = await app.evaluate(() => window.eventLab.requests.length);
  await open();
  await wait(() => app.evaluate(count => window.eventLab.requests.length > count, heldCount), 'Held request missing');
  await reset('?cancel');
  await wait(() => app.evaluate(() => window.eventLab.requests.at(-1).signal.aborted), 'Source navigation did not abort the window request');
  assert.equal(await app.evaluate(() => window.eventLab.requests.at(-1).discard()), false);
  console.log('window-cancel=passed (navigation, AbortSignal, stale response)');

  await select('#event-handler','top');
  await reset('?iframe');
  const iframeOwner = pages().find(page => page.url().includes('/event-fixture.html?iframe'));
  await wait(() => iframeOwner.frame({name:'event-frame'})?.url().includes('?frame'), 'Iframe source missing');
  const originalFrame = iframeOwner.frame({name:'event-frame'});
  const iframeHeld = await app.evaluate(() => window.eventLab.requests.length);
  await originalFrame.locator('#open').click();
  await wait(() => app.evaluate(before => window.eventLab.requests.length > before, iframeHeld), 'Iframe window request missing');
  await originalFrame.goto(new URL('event-fixture.html?frame-replaced',base).href);
  await wait(() => app.evaluate(() => window.eventLab.requests.at(-1).signal.aborted), 'Iframe navigation did not cancel its pending window');
  assert.ok((await state()).url.endsWith('?iframe'), 'Iframe cancellation should not require main-document navigation');
  console.log('iframe-window-cancel=passed (original opener document lifetime)');

  // Completely unhandled top requests are closed instead of leaking hidden windows.
  await select('#event-handler','none');
  const unhandled = await count('top');
  await open();
  await wait(async () => await count('top') > unhandled, 'Unhandled request never reached top');
  await wait(() => execute('window.lastPopup === null || window.lastPopup.closed'), 'Unhandled native window was not discarded');

  // Nested documents have separate dispatches; local stops still reach top.
  await reset('?nested');
  const parentPage = pages().find(page => page.url().includes('/event-fixture.html?nested'));
  await wait(() => parentPage.locator('#nested-child').evaluate(async element => {
    const value = await element.getNavigationState(); return value.url.includes('?leaf') && !value.isLoading;
  }), 'Nested child missing');
  const leaf = pages().find(page => page.url().includes('/event-fixture.html?leaf'));
  assert.ok(parentPage && leaf);
  await select('#event-handler','top');
  await select('#window-action','discard');
  for (const mode of ['bubble','stop','immediate','claim']) {
    await parentPage.evaluate(mode => { window.childMode = mode; }, mode);
    const before = await count('top');
    await leaf.locator('#open').click();
    if (mode !== 'claim') await wait(async () => await count('top') > before, `${mode} did not reach top`);
    else { await app.waitForTimeout(250); assert.equal(await count('top'),before); }
  }
  console.log('nested=passed (bubble, stop, immediate, claim)');

  await reset('?dialogs');
  const guest = pages().find(page => page.url().includes('/event-fixture.html?dialogs'));
  for (const [selector,result] of [['#confirm',true],['#prompt','event answer'],['#alert','done']]) {
    await guest.locator(selector).click();
    await wait(() => guest.evaluate(result => window.dialogResult === result, result), 'Native JavaScript dialog result mismatch');
    assert.equal(await app.evaluate(() => window.eventLab.requests.at(-1).dismiss()),false);
    assert.equal(await app.evaluate(() => window.eventLab.requests.at(-1).signal.aborted),false);
  }
  console.log('dialogs=passed (alert, confirm, prompt, one-shot, normal completion)');

  await select('#event-handler','element');
  await app.evaluate(() => { window.eventLab.holdDialog = true; });
  const dialogsBefore = await app.evaluate(() => window.eventLab.requests.length);
  const pendingClick = guest.locator('#confirm').click().catch(() => {});
  await wait(() => app.evaluate(before => window.eventLab.requests.length > before, dialogsBefore), 'Held dialog missing');
  await reset('?dialog-cancel');
  await wait(() => app.evaluate(() => window.eventLab.requests.at(-1).signal.aborted), 'Source navigation did not cancel the held dialog');
  assert.equal(await app.evaluate(() => window.eventLab.requests.at(-1).accept()), false);
  await pendingClick;
  await app.evaluate(() => { window.eventLab.holdDialog = false; });
  await select('#event-handler','top');
  console.log('dialog-cancel=passed (native cancellation, retained signal, stale response)');

  // Original POST navigation must survive pending-window adoption.
  await select('#window-action','attach');
  const postUrl = `http://127.0.0.1:${server.address().port}/original`;
  const postGuest = pages().find(page => page.url().includes('/event-fixture.html?dialog-cancel'));
  await postGuest.evaluate(url => {
    const form = document.createElement('form'); form.method='POST'; form.action=url; form.target='_blank';
    const input = document.createElement('input'); input.name='proof'; input.value='original-body'; form.append(input); document.body.append(form); form.submit();
  },postUrl);
  await wait(async () => postBody?.method === 'POST', 'Original POST request never reached server');
  assert.equal(postBody.body,'proof=original-body');
  await wait(() => app.locator('#event-windows explorer').last().evaluate(async element => {
    const value = await element.getNavigationState(); return value.url.startsWith('http://127.0.0.1:') && !value.isLoading;
  }), 'POST window did not finish its original navigation');
  console.log('post=passed (original body and method preserved)');

  // Title changes are not fake navigation commits; state notifications can be delegated.
  const commits = await count('state','loadcommit');
  await execute("document.title = 'Changed event title'; true");
  await wait(async () => await count('state','titlechange') > 0, 'titlechange missing');
  assert.equal(await count('state','loadcommit'), commits);
  await app.locator('#event-source').evaluate(element => element.setZoomFactor(1.25));
  const zoom = await app.evaluate(() => window.eventLab.events.findLast(event => event.type === 'zoomchange'));
  assert.equal(zoom.detail.oldFactor,1); assert.equal(zoom.detail.newFactor,1.25);
  assert.equal(zoom.bubbles,true); assert.equal(zoom.composed,true); assert.equal(zoom.cancelable,false);
  console.log('state=passed (title independent of commit, zoom factors, delegation flags)');

  await select('#window-action','discard');
  await app.evaluate(url => {
    const host = document.createElement('div'); host.id = 'event-shadow-host';
    const root = host.attachShadow({mode:'open'});
    const inner = document.createElement('explorer'); inner.id = 'event-shadow-source';
    inner.navigation = 'allow'; inner.dialogs = 'emit'; inner.style.cssText='display:block;width:600px;height:200px';
    root.append(inner); document.body.append(host);
    document.addEventListener('newwindow', event => {
      if (event.target === host) window.eventLab.shadowProof = {target:event.target.id, path:event.composedPath()[0].id};
    });
    inner.src = url; window.eventLab.shadowSource = inner;
  },new URL('event-fixture.html?shadow',base).href);
  await wait(() => app.evaluate(async () => {
    const value = await window.eventLab.shadowSource.getNavigationState(); return value.url.endsWith('?shadow') && !value.isLoading;
  }), 'Shadow-root source did not load');
  const shadowGuest = pages().find(page => page.url().endsWith('/event-fixture.html?shadow'));
  await shadowGuest.locator('#open').click();
  await wait(() => app.evaluate(() => !!window.eventLab.shadowProof), 'Composed request did not leave its shadow root');
  assert.deepEqual(await app.evaluate(() => window.eventLab.shadowProof), {target:'event-shadow-host', path:'event-shadow-source'});
  console.log('shadow=passed (composed path and standard retargeting)');
  console.log(JSON.stringify({eventModel:true, originalWindow:true, nestedFallback:true, dialogs:true, cancellation:true, post:true}));
} catch (error) {
  console.error(error.stack);
  console.error(await Promise.race([
    app.evaluate(() => ({events:window.eventLab.events.slice(-8), error:window.eventLab.error, response:window.eventLab.response, requests:window.eventLab.requests.length})),
    new Promise(resolve => setTimeout(() => resolve('diagnostic timed out'), 2000)),
  ]).catch(() => 'diagnostic unavailable'));
  throw error;
} finally {
  await browser.close();
  server.closeAllConnections?.();
  await new Promise(resolve => server.close(resolve));
}

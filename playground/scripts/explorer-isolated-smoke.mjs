// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
import assert from 'node:assert/strict';
import http from 'node:http';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../ui/package.json', import.meta.url));
const { chromium } = require('playwright-core');
let origin;
const server = http.createServer((request, response) => {
  const url = new URL(request.url, origin);
  response.setHeader('Content-Type', 'text/html');
  if (url.pathname === '/callback') {
    response.end(`<!doctype html><title>OAuth callback</title><body>Local callback</body><script>window.initialTopSelf=top===self;window.initialCamera=(document.permissionsPolicy||document.featurePolicy).allowsFeature('camera')</script>`);
  } else if (url.pathname === '/oauth') {
    const destination = url.searchParams.get('other') ? origin.replace('localhost', '127.0.0.2') : origin;
    response.end(`<!doctype html><script>
      setTimeout(() => {
        const report={oauth:true,topIsParent:top===parent,activated:navigator.userActivation.hasBeenActive};
        try { top.location.href=${JSON.stringify(destination + '/callback?run=' + url.searchParams.get('run'))}; report.error=null; }
        catch(error) { report.error=error.name; }
        parent.postMessage(report,'*');
      },100);
    </script>`);
  } else {
    const child = origin.replace('localhost', '127.0.0.1') + '/oauth' + url.search;
    response.end(`<!doctype html><script>
      window.initialTopSelf=top===self;window.reports=[];addEventListener('message',event=>{if(event.data?.oauth)reports.push(event.data)});
    </script><iframe src=${JSON.stringify(child)}></iframe>`);
  }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
origin = `http://localhost:${server.address().port}`;
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257}`);
const pages = () => browser.contexts().flatMap(context => context.pages());
async function until(check, message) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const result = await check();
    if (result) return result;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  throw new Error(message);
}
async function evaluate(url, expression) {
  const targets = await (await fetch(`http://127.0.0.1:${process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257}/json/list`)).json();
  const target = targets.find(item => item.url === url);
  assert.ok(target, `Missing fixture ${url}`);
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  try {
    return await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Fixture evaluation timed out')), 15000);
      socket.addEventListener('message', event => {
        const reply = JSON.parse(event.data);
        if (reply.id !== 1) return;
        clearTimeout(timeout);
        if (reply.error || reply.result?.exceptionDetails)
          reject(new Error(JSON.stringify(reply.error || reply.result.exceptionDetails)));
        else resolve(reply.result.result.value);
      });
      socket.send(JSON.stringify({ id: 1, method: 'Runtime.evaluate', params: {
        expression, returnByValue: true, awaitPromise: true,
      } }));
    });
  } finally { socket.close(); }
}
try {
  const app = await until(() => pages().find(page => page.url().includes('/index.html')), 'Missing Playground');
  const appUrl = app.url();
  assert.deepEqual(await app.evaluate(() => {
    const element=document.createElement('explorer');
    const values=[element.isolated];
    element.setAttribute('isolated','true');values.push(element.isolated);
    element.setAttribute('isolated','false');values.push(element.isolated);
    element.isolated=false;values.push(element.hasAttribute('isolated'));
    return values;
  }),[false,true,true,false]);
  let run = 0;
  async function mount(isolated, other = false) {
    const key = String(++run);
    await app.evaluate(({url,isolated}) => {
      window.oauthExplorer?.remove();
      const explorer = document.createElement('explorer');
      explorer.isolated=isolated;
      explorer.allow="camera 'none'; microphone 'none'";
      explorer.navigation='allow';
      explorer.style.cssText='display:block;width:800px;height:500px';
      explorer.src=url;
      window.oauthExplorer=explorer;
      document.body.append(explorer);
    }, {url:origin + '/login?run=' + key + (other ? '&other=1' : ''),isolated});
    return until(() => pages().find(page => page.url().startsWith(origin) && new URL(page.url()).searchParams.get('run') === key), 'Missing login Explorer');
  }
  // Existing composed ancestry still targets the app and rejects the framebust.
  let page = await mount(false);
  await until(async () => (await page.evaluate(() => window.reports))?.length, 'Missing embedded callback rejection');
  assert.equal(await page.evaluate(() => initialTopSelf),false);
  assert.deepEqual(await page.evaluate(() => reports[0]), {oauth:true,topIsParent:false,activated:false,error:'SecurityError'});
  assert.equal(app.url(),appUrl);
  // Isolated: a cross-origin QR frame can redirect its own login page back to
  // that page's origin, without activation and without navigating the app.
  page = await mount(true);
  await until(() => page.url().startsWith(origin + '/callback?'), 'OAuth callback did not navigate the isolated Explorer');
  assert.deepEqual(await page.evaluate(() => ({initialTopSelf,initialCamera,topSelf:top===self,parentSelf:parent===self,ancestors:[...location.ancestorOrigins],camera:(document.permissionsPolicy || document.featurePolicy).allowsFeature('camera')})),
    {initialTopSelf:true,initialCamera:false,topSelf:true,parentSelf:true,ancestors:[],camera:false});
  assert.equal(app.url(),appUrl);
  assert.deepEqual(await page.evaluate(() => {
    document.cookie='isolated-oauth-session=present; Path=/; SameSite=None; Secure';
    localStorage.setItem('isolated-oauth-session','present');
    return {cookie:document.cookie.includes('isolated-oauth-session=present'),storage:localStorage.getItem('isolated-oauth-session')==='present'};
  }),{cookie:true,storage:true});
  // Explicit WindowProxy messaging remains usable without exposing parent/top.
  await page.evaluate(() => addEventListener('message',event=>{
    if(event.data?.oauthPing) event.source.postMessage({oauthPong:true},event.origin);
  }));
  assert.equal(await app.evaluate(() => new Promise(resolve=>{
    const timer=setTimeout(()=>resolve(false),10000);
    const listener=event=>{if(event.data?.oauthPong){clearTimeout(timer);removeEventListener('message',listener);resolve(event.source===oauthExplorer.contentWindow)}};
    addEventListener('message',listener);oauthExplorer.postMessage({oauthPing:true},'*');
  })),true);
  // Attribute writes cannot mutate a live native policy; a transferred page
  // must be reattached with exactly the same isolated mode.
  await app.evaluate(() => {oauthExplorer.isolated=false});
  assert.equal(await page.evaluate(() => top===self),true);
  const documentId = await app.evaluate(async () => (await oauthExplorer.getNavigationState()).documentId);
  await app.evaluate(async () => {window.oauthHandle=await oauthExplorer.detach()});
  assert.equal(await app.evaluate(async () => {
    try {await oauthExplorer.attach(oauthHandle);return 'allowed'} catch(error){return error.name}
  }),'SecurityError');
  await app.evaluate(async () => {oauthExplorer.isolated=true;await oauthExplorer.attach(oauthHandle)});
  assert.equal(await app.evaluate(async () => (await oauthExplorer.getNavigationState()).documentId),documentId);
  assert.equal(await page.evaluate(() => top===self && parent===self),true);
  // Independent mode does not disable Chromium's cross-origin/user-activation guard.
  page = await mount(true,true);
  await until(async () => (await page.evaluate(() => window.reports))?.length, 'Missing cross-origin navigation rejection');
  assert.deepEqual(await page.evaluate(() => reports[0]),{oauth:true,topIsParent:true,activated:false,error:'SecurityError'});
  assert.equal(app.url(),appUrl);
  const rootUrl = new URL('history-a.html?isolated-nested-root',appUrl).href;
  await app.evaluate(url => {
    oauthExplorer.remove();
    const explorer=document.createElement('explorer');explorer.isolated=true;
    explorer.allow="camera 'none'";
    explorer.style.cssText="display:block;width:800px;height:500px";
    explorer.navigation='allow';explorer.src=url;
    window.oauthExplorer=explorer;document.body.append(explorer);
  },rootUrl);
  const root = await until(()=>pages().find(page=>page.url()===rootUrl),'Missing isolated app fixture');
  await until(async()=>!(await evaluate(rootUrl,"(document.permissionsPolicy||document.featurePolicy).allowsFeature('camera')")),'Initial root camera denial did not apply');
  assert.deepEqual(await evaluate(rootUrl,"({topSelf:top===self,camera:(document.permissionsPolicy||document.featurePolicy).allowsFeature('camera')})"),
    {topSelf:true,camera:false},'Isolated app root must retain its owner permissions');
  const nestedUrl = new URL('history-b.html?isolated-nested-child',appUrl).href;
  await evaluate(rootUrl,`{const child=document.createElement('explorer');child.style.cssText='display:block;width:600px;height:300px';child.navigation='allow';child.src=${JSON.stringify(nestedUrl)};document.body.append(child);}`);
  const nested = await until(()=>pages().find(page=>page.url()===nestedUrl),'Missing nested composed Explorer');
  await until(async()=>await evaluate(nestedUrl,"top.parent===top && !(document.permissionsPolicy||document.featurePolicy).allowsFeature('camera')"),'Nested isolated boundary or permissions did not initialize');
  assert.deepEqual(await evaluate(nestedUrl,"({parentTop:parent===top,topParentSelf:top.parent===top,topSelf:top===self,ancestors:location.ancestorOrigins.length,camera:(document.permissionsPolicy||document.featurePolicy).allowsFeature('camera')})"),
    {parentTop:true,topParentSelf:true,topSelf:false,ancestors:1,camera:false});
  assert.equal(await evaluate(rootUrl,'top===self&&parent===self'),true);
  console.log(JSON.stringify({isolated_oauth_callback:true,host_preserved:true,composed_default:true,permissions_preserved:true,session_storage:true,explicit_messages:true,immutable_transfer_policy:true,cross_origin_guard:true,nested_boundary:true}));
} finally {
  await browser.close();
  server.closeAllConnections();
  await new Promise(resolve=>server.close(resolve));
}

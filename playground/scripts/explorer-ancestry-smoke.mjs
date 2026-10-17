import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../ui/package.json', import.meta.url));
const { chromium } = require('playwright-core');
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257);
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
const pages = () => browser.contexts().flatMap(context => context.pages());
const app = pages().find(page => page.url().includes('/index.html'));
assert.ok(app, 'Playground main window is missing');
const base = new URL('./', app.url()).href;
const appOrigin = await app.evaluate(() => location.origin);
assert.equal(await app.evaluate(() => self.origin),appOrigin,'Application documents must retain their normal tuple security origin');
async function wait(check, message) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    if (await check()) return;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  throw new Error(message);
}
// Use CDP only to operate our fixtures' main world; public executeJavaScript
// deliberately runs in an isolated world.
async function evaluate(url, expression) {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
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
  await app.evaluate(async base => {
    window.ancestryMessages = [];
    window.addEventListener('message', event => {
      if (!event.data?.ancestryTest) return;
      window.ancestryMessages.push({ ...event.data, trusted: event.isTrusted,
        standard: event instanceof MessageEvent, origin: event.origin,
        direct: event.source === window.ancestryExplorer.contentWindow,
        ports: event.ports.length });
      event.source?.postMessage({ ancestryReply: event.data.ancestryTest }, event.origin);
      event.ports[0]?.postMessage('port-ack');
    });
    const explorer = document.createElement('explorer');
    explorer.navigation = 'allow';
    explorer.style.cssText = 'display:block;width:800px;height:500px';
    explorer.src = new URL('history-a.html?ancestry-root', base).href;
    window.ancestryExplorer = explorer;
    const loaded = new Promise(resolve => explorer.addEventListener('loadstop', resolve, { once: true }));
    document.body.append(explorer);
    await loaded;
  }, base);
  const firstUrl = new URL('history-a.html?ancestry-root', base).href;
  const first = await evaluate(firstUrl, `(() => {
    window.ancestryReplies=[];
    window.addEventListener('message',event=>{
      if(event.data?.ancestryReply) window.ancestryReplies.push({reply:event.data.ancestryReply,
        sourceIsParent:event.source===parent,trusted:event.isTrusted});
    });
    const channel=new MessageChannel();
    channel.port1.onmessage=event=>window.ancestryPortAck=event.data;
    const returned=parent.postMessage({ancestryTest:'parent',map:new Map([['x',42n]])},location.origin,[channel.port2]);
    top.postMessage({ancestryTest:'top'},{targetOrigin:location.origin});
    tuffite.postMessage({ancestryTest:'tuffite'},{targetOrigin:location.origin});
    tuffite.postMessage({ancestryTest:'tuffite-positional'},location.origin,[]);
    return {parentSelf:parent===window,topSelf:top===window,parentTop:parent===top,
      parentHasPost:typeof parent.postMessage,returned:returned===undefined,ancestorOrigins:[...location.ancestorOrigins]};
  })()`);
  assert.deepEqual(first, {parentSelf:false,topSelf:false,parentTop:true,parentHasPost:'function',returned:true,ancestorOrigins:[appOrigin]});
  await wait(async () => (await app.evaluate(() => ancestryMessages.length)) >= 4, 'Native parent/top messages did not arrive').catch(async error => {
    console.log(JSON.stringify({app:await app.evaluate(()=>({origin:location.origin,messages:ancestryMessages})),
      guest:await evaluate(firstUrl,`(() => {let parentURL;try{parentURL=parent.location.href;}catch(error){parentURL=error.name;}return {origin:location.origin,parentURL,replies:ancestryReplies};})()`)}));
    throw error;
  });
  const direct = await app.evaluate(() => ancestryMessages);
  for (const event of direct) {
    assert.equal(event.trusted,true); assert.equal(event.standard,true);
    assert.equal(event.direct,true); assert.equal(event.origin,appOrigin);
  }
  await wait(async () => (await evaluate(firstUrl,'ancestryReplies.length===4 && ancestryPortAck==="port-ack"')), 'Native source replies or transferred port failed');
  assert.equal(await evaluate(firstUrl,'ancestryReplies.every(event=>event.sourceIsParent&&event.trusted)'),true);
  assert.equal(await app.evaluate(() => ancestryExplorer.postMessage({ancestryReply:'element'},{targetOrigin:location.origin})===undefined),true);
  await wait(async () => (await evaluate(firstUrl,'ancestryReplies.some(event=>event.reply==="element")')), 'Explorer.postMessage did not reach guest');
  await evaluate(firstUrl, `window.addEventListener('message',async event=>{
    if(event.data?.type!=='ancestry-handle')return;
    const target=document.createElement('explorer');target.navigation='allow';document.body.append(target);
    try {
      await target.attach(event.data.handle);
      const state=await target.getNavigationState();
      event.ports[0].postMessage({ok:true,documentId:state.documentId,
        sourceIsParent:event.source===parent,trusted:event.isTrusted,handleState:event.data.handle.state});
    } catch(error) {event.ports[0].postMessage({ok:false,error:String(error)});}
    finally {target.remove();event.ports[0].close();}
  })`);
  const moved = await app.evaluate(async base => {
    const source=document.createElement('explorer');source.navigation='allow';
    source.src=new URL('history-a.html?ancestry-handle',base).href;
    const loaded=new Promise(resolve=>source.addEventListener('loadstop',resolve,{once:true}));
    document.body.append(source);await loaded;
    const original=await source.getNavigationState();const handle=await source.detach();
    const channel=new MessageChannel();
    try {
      const reply=new Promise((resolve,reject)=>{
        const timer=setTimeout(()=>reject(new Error('Handle transfer acknowledgement timed out')),15000);
        channel.port1.onmessage=event=>{clearTimeout(timer);resolve(event.data);};
      });
      ancestryExplorer.postMessage({type:'ancestry-handle',handle},location.origin,[handle,channel.port2]);
      return {original:original.documentId,sender:handle.state,reply:await reply};
    } finally {source.remove();channel.port1.close();}
  },base);
  assert.equal(moved.sender,'transferred'); assert.equal(moved.reply.ok,true);
  assert.equal(moved.reply.documentId,moved.original);
  assert.equal(moved.reply.sourceIsParent,true); assert.equal(moved.reply.trusted,true);
  assert.equal(moved.reply.handleState,'transferred');
  await evaluate(firstUrl, `(async()=>{
    const iframe=document.createElement('iframe');
    iframe.srcdoc='<script>parent.parent.postMessage({ancestryTest:"iframe",parentSelf:parent===window,topIsApp:top===parent.parent},"*")<\/script>';
    document.body.append(iframe);
    const child=document.createElement('explorer');child.navigation='allow';
    child.src=new URL('history-b.html?ancestry-nested',location.href).href;
    window.nestedAncestryExplorer=child;
    const loaded=new Promise(resolve=>child.addEventListener('loadstop',resolve,{once:true}));
    document.body.append(child);await loaded;return true;
  })()`);
  const nestedUrl = new URL('history-b.html?ancestry-nested', base).href;
  assert.deepEqual(await evaluate(nestedUrl, `(() => {
    top.postMessage({ancestryTest:'nested'},'*');
    return {parentSelf:parent===window,topSelf:top===window,parentTop:parent===top,parentTopIsTop:parent.top===top,ancestorOrigins:[...location.ancestorOrigins]};
  })()`), {parentSelf:false,topSelf:false,parentTop:false,parentTopIsTop:true,ancestorOrigins:[appOrigin,appOrigin]});
  await wait(async()=> (await app.evaluate(()=>ancestryMessages.some(event=>event.ancestryTest==='iframe')&&ancestryMessages.some(event=>event.ancestryTest==='nested'))),'Nested Explorer/iframe top messages did not arrive');
  const iframe = await app.evaluate(()=>ancestryMessages.find(event=>event.ancestryTest==='iframe'));
  assert.equal(iframe.parentSelf,false); assert.equal(iframe.topIsApp,true);
  // Detached pages lose host ancestry; attach restores it without replacing the document.
  const original = await app.evaluate(()=>ancestryExplorer.getNavigationState());
  await app.evaluate(async()=>{window.ancestryHandle=await ancestryExplorer.detach();});
  assert.deepEqual(await evaluate(firstUrl,'({parentSelf:parent===window,topSelf:top===window})'),{parentSelf:true,topSelf:true});
  assert.deepEqual(await evaluate(firstUrl,'[...location.ancestorOrigins]'),[]);
  assert.equal(await evaluate(nestedUrl,'parent===top'),true,'Nested top must follow the detached subtree root');
  assert.deepEqual(await evaluate(nestedUrl,'[...location.ancestorOrigins]'),[appOrigin]);
  await app.evaluate(async()=>{await ancestryExplorer.attach(ancestryHandle);});
  assert.deepEqual(await evaluate(firstUrl,'({parentSelf:parent===window,topSelf:top===window})'),{parentSelf:false,topSelf:false});
  assert.equal(await evaluate(nestedUrl,'parent!==top && parent.top===top'),true,'Nested top must follow the reattached host');
  assert.deepEqual(await evaluate(nestedUrl,'[...location.ancestorOrigins]'),[appOrigin,appOrigin]);
  assert.equal((await app.evaluate(()=>ancestryExplorer.getNavigationState())).documentId,original.documentId);
  console.log(JSON.stringify({ancestry:true,nativeSource:true,structuredMessaging:true,nested:true,detachAttach:true}));
} finally {
  await app.evaluate(()=>window.ancestryExplorer?.remove()).catch(()=>{});
  await browser.close();
}

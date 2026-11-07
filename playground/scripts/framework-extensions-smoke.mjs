// Uses the packaged application's actual native profile manager and Rust backends.
import assert from 'node:assert/strict';
const port = process.env.TUFFITE_PLAYGROUND_CDP_PORT;
const endpoint = `http://127.0.0.1:${port}`;
const connections = [];
async function until(check, message) {
  const deadline = Date.now() + 45000;
  while (Date.now() < deadline) {
    const result = await check();
    if (result) return result;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  throw new Error(message);
}
async function connect(exclude = []) {
  const target = await until(async () => {
    const targets = await (await fetch(`${endpoint}/json/list`)).json();
    return targets.find(t => t.type === 'page' && t.url.includes('/framework.html') && !exclude.includes(t.id));
  }, 'Framework content target missing');
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { socket.addEventListener('open', resolve, { once: true }); socket.addEventListener('error', reject, { once: true }); });
  let sequence = 0;
  const api = {
    id: target.id,
    async command(method, params) {
      const id = ++sequence;
      return new Promise((resolve, reject) => {
        function cleanup() { clearTimeout(timer); socket.removeEventListener('message', receive); socket.removeEventListener('close', closed); }
        function closed() { cleanup(); reject(new Error('CDP target closed')); }
        const timer = setTimeout(() => { cleanup(); reject(new Error(`CDP ${method} timed out`)); }, 40000);
        function receive(event) {
          const value = JSON.parse(event.data); if (value.id !== id) return;
          cleanup();
          if (value.error) reject(new Error(JSON.stringify(value.error)));
          else resolve(value.result);
        }
        socket.addEventListener('message', receive);
        socket.addEventListener('close', closed, { once: true });
        socket.send(JSON.stringify({ id, method, params }));
      });
    },
    async eval(expression) {
      const value = await this.command('Runtime.evaluate', { expression: `(async () => { ${expression} })()`, awaitPromise: true, returnByValue: true });
      if (value.exceptionDetails) throw new Error(JSON.stringify(value.exceptionDetails));
      return value.result?.value;
    },
    close() { socket.close(); },
  };
  connections.push(api);
  await until(() => api.eval('return Boolean(window.frameworkLab);'), 'Framework lab initialization failed');
  return api;
}
try {
  const api = await connect();
  const result = await api.eval('return await frameworkLab.run();');
  assert.equal(result.resource.profile, 'default');
  assert.equal(result.resource.context, 'playground application');
  assert.equal(result.consumed, 204);
  assert.equal(result.discarded, true);
  assert.equal(result.missing, 404);
  assert.equal(result.image, 96);
  await until(() => api.eval("return frameworkLab.events.some(e => e.channel === 'playground.framework.action');"), 'Consumed action event missing');
  assert.equal(await api.eval('return document.querySelector("#image-dialog").open;'), true);
  await api.eval('document.querySelector("#image-dialog").close();');
  await api.command('Page.bringToFront', {});
  const modifier = await api.eval("return navigator.platform.includes('Mac') ? 4 : 2;");
  async function press(code, key) {
    await api.command('Input.dispatchKeyEvent', { type: 'rawKeyDown', key, code: `Key${key}`, windowsVirtualKeyCode: code, modifiers: modifier | 8 });
    await api.command('Input.dispatchKeyEvent', { type: 'keyUp', key, code: `Key${key}`, windowsVirtualKeyCode: code, modifiers: modifier | 8 });
  }
  await press(80, 'P');
  await until(() => api.eval("return frameworkLab.events.some(e => e.channel === 'playground.framework.shortcut' && e.data.startsWith('Application:'));"), 'Application shortcut missing');
  await press(75, 'K');
  await until(() => api.eval("return frameworkLab.events.some(e => e.channel === 'playground.framework.shortcut' && e.data.includes('callback'));"), 'Dedicated shortcut callback missing');
  assert.equal(await api.eval("return (await fetch('/framework/shortcuts/window')).status;"), 204);
  await api.eval('frameworkLab.events.length = 0;');
  await press(80, 'P');
  await until(() => api.eval("return frameworkLab.events.some(e => e.channel === 'playground.framework.shortcut' && e.data.startsWith('Window('));"), 'Window shortcut did not take priority');
  assert.equal(await api.eval("return frameworkLab.events.some(e => e.data.startsWith('Application:'));"), false);
  const location = await api.eval('return window.location.href;');
  await api.command('Page.navigate', { url: 'playground://main/framework/discard' });
  assert.equal(await api.eval('return window.location.href;'), location);
  console.log('Framework extensions passed: contexts, metadata, image/file resources, consumed actions, cancellation, events, application/window shortcuts and callbacks.');
} finally {
  for (const api of connections) api.close();
}

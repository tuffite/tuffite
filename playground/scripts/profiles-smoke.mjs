// Uses the packaged application's actual native profile manager and Rust backends.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
const port = process.env.TUFFITE_PLAYGROUND_CDP_PORT;
const endpoint = `http://127.0.0.1:${port}`;
const cookieUrl = 'https://profiles.invalid/';
const connections = [];
let restarted;
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
    return targets.find(t => t.type === 'page' && t.url.includes('/profiles.html') && !exclude.includes(t.id));
  }, 'Profile content target missing');
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
  await until(() => api.eval('return Boolean(window.profileLab);'), 'Profile lab initialization failed');
  await api.eval("window.api = profileLab.api; window.native = { window: profileLab.nativeWindow }; window.received = []; window.tuffite.addEventListener('channel', e => received.push(e.detail.channel));");
  return api;
}
try {
  const personal = await connect();
  const original = await personal.eval('return profileLab.identity;');
  assert.equal(original.profile, 'default');
  assert.equal(original.application, 'playground application');
  const windowId = await personal.eval('return (await native.window.current()).id.toString();');
  await personal.eval("await api.profiles.create('work', 'Work'); await api.profiles.create('dormant', 'Dormant'); localStorage.setItem('isolation', 'personal');");
  assert.equal((await personal.command('Network.setCookie', { name: 'identity', value: 'personal', url: cookieUrl, expires: Date.now() / 1000 + 3600 })).success, true);
  const emptyWindow = await personal.eval('return (await api.profiles.createWindow()).toString();');
  await personal.eval(`await api.profiles.closeWindow(BigInt(${JSON.stringify(emptyWindow)}));`);
  assert.deepEqual(await personal.eval('return JSON.parse(await api.profiles.list()).map(p => [p.id, p.loaded]);'), [['default', true], ['dormant', false], ['work', false]]);
  assert.equal(await personal.eval('return await api.browserDiagnostics.increment();'), 1);
  assert.equal(await personal.eval('return await api.profiles.applicationState();'), 1);
  // Opening the same profile twice must share its backend; preparing views does
  // not change the source window, and invalid navigation preserves that content.
  await personal.eval("let failed = false; try { await api.profiles.prepare('work', 'not a URL'); } catch { failed = true; } if (!failed) throw Error('invalid URL accepted');");
  assert.equal(await personal.eval('return (await native.window.current()).id.toString();'), windowId);
  let prepared = await personal.eval("return await api.profiles.prepare('work', location.href);");
  const extra = await connect([personal.id]);
  const extraIdentity = await extra.eval('return profileLab.identity;');
  const extraWindow = await personal.eval('return (await api.profiles.createWindow()).toString();');
  assert.equal(await personal.eval(`return await api.profiles.attach(BigInt(${JSON.stringify(extraWindow)}), ${JSON.stringify(prepared)});`), '');
  assert.equal(await extra.eval('return (await native.window.current()).id.toString();'), extraWindow);
  assert.equal(await personal.eval('return (await native.window.current()).id.toString();'), windowId);
  assert.equal(await personal.eval(`return (await native.window.list()).some(w => w.id.toString() === ${JSON.stringify(extraWindow)});`), false);
  await personal.eval(`let denied = false; try { await native.window.focus(BigInt(${JSON.stringify(extraWindow)})); } catch (error) { denied = error.code === 'access_denied'; } if (!denied) throw Error('cross-profile renderer window control accepted');`);
  prepared = await personal.eval("return await api.profiles.prepare('work', location.href);");
  const work = await connect([personal.id, extra.id]);
  const workIdentity = await work.eval('return profileLab.identity;');
  assert.equal(workIdentity.context, extraIdentity.context);
  await personal.eval(`await api.profiles.closeWindow(BigInt(${JSON.stringify(extraWindow)}));`);
  assert.equal(workIdentity.profile, 'work');
  assert.notEqual(workIdentity.context, original.context);
  assert.equal(await work.eval("return localStorage.getItem('isolation');"), null);
  assert.equal((await work.command('Network.getCookies', { urls: [cookieUrl] })).cookies.some(c => c.name === 'identity'), false);
  assert.equal(await work.eval('return await api.browserDiagnostics.increment();'), 1);
  assert.equal(await work.eval('return await api.profiles.applicationState();'), 2);
  await work.eval("localStorage.setItem('isolation', 'work');");
  assert.equal((await work.command('Network.setCookie', { name: 'identity', value: 'work', url: cookieUrl, expires: Date.now() / 1000 + 3600 })).success, true);
  assert.equal((await work.command('Network.setCookie', { name: 'sessionOnly', value: 'work', url: cookieUrl })).success, true);
  assert.equal((await work.command('Network.getCookies', { urls: [cookieUrl] })).cookies.find(c => c.name === 'identity')?.value, 'work');
  await personal.eval("await api.commands.doSomething('profile-event');");
  assert.equal(await personal.eval("return received.includes('playground:command');"), true);
  assert.equal(await work.eval("return received.includes('playground:command');"), false);
  const old = await personal.eval(`return await api.profiles.attach(BigInt(${JSON.stringify(windowId)}), ${JSON.stringify(prepared)});`);
  assert.equal(await work.eval('return (await native.window.current()).id.toString();'), windowId);
  await work.eval(`let rejected = false; try { await api.profiles.close(${JSON.stringify(original.context)}); } catch { rejected = true; } if (!rejected) throw Error('referenced profile closed');`);
  const detachedWork = await work.eval(`return await api.profiles.attach(BigInt(${JSON.stringify(windowId)}), ${JSON.stringify(old)});`);
  assert.equal(await personal.eval('return await api.browserDiagnostics.increment();'), 2);
  assert.equal(await personal.eval("return localStorage.getItem('isolation');"), 'personal');
  await personal.eval(`await api.profiles.releaseView(${JSON.stringify(detachedWork)}); await api.profiles.close(${JSON.stringify(workIdentity.context)});`);
  await personal.eval(`let rejected = false; try { await api.profiles.close(${JSON.stringify(workIdentity.context)}); } catch { rejected = true; } if (!rejected) throw Error('stale context handle accepted');`);
  const reopenedView = await personal.eval("return await api.profiles.prepare('work', location.href);");
  const reopened = await connect([personal.id, work.id, extra.id]);
  const reopenedIdentity = await reopened.eval('return profileLab.identity;');
  assert.notEqual(reopenedIdentity.context, workIdentity.context);
  assert.equal(await reopened.eval('return await api.browserDiagnostics.increment();'), 1);
  assert.equal(await reopened.eval("return localStorage.getItem('isolation');"), 'work');
  const cookies = (await reopened.command('Network.getCookies', { urls: [cookieUrl] })).cookies;
  assert.equal(cookies.find(c => c.name === 'identity')?.value, 'work');
  assert.equal(cookies.some(c => c.name === 'sessionOnly'), false);
  await personal.eval(`await api.profiles.releaseView(${JSON.stringify(reopenedView)}); await api.profiles.close(${JSON.stringify(reopenedIdentity.context)});`);
  const remaining = await personal.eval('return JSON.parse(await api.profiles.list());');
  assert.equal(remaining.find(p => p.id === 'dormant').loaded, false);
  assert.equal(remaining.find(p => p.id === 'work').loaded, false);
  // Exercise a real cold process launch against the populated profile catalog.
  // Normal last-window closure also runs both Rust shutdown scopes.
  await personal.eval('native.window.close(); return true;').catch(error => {
    if (!String(error).includes('target closed')) throw error;
  });
  const originalPid = Number(process.env.TUFFITE_PLAYGROUND_PID);
  await until(() => { try { process.kill(originalPid, 0); return false; } catch { return true; } }, 'Original application did not shut down');
  const command = JSON.parse(process.env.TUFFITE_PLAYGROUND_COMMAND);
  const launched = Date.now();
  restarted = spawn(command[0], command.slice(1), { cwd: process.env.TUFFITE_PLAYGROUND_CWD, stdio: 'ignore', windowsHide: true });
  const cold = await until(async () => {
    if (restarted.exitCode !== null) throw new Error(`Cold launch exited: ${restarted.exitCode}`);
    try { await fetch(`${endpoint}/json/version`); return true; } catch { return false; }
  }, 'Cold launch CDP endpoint missing').then(() => connect());
  assert.deepEqual(await cold.eval('return JSON.parse(await api.profiles.list()).map(p => [p.id, p.loaded]);'), [['default', true], ['dormant', false], ['work', false]]);
  assert.equal(await cold.eval('return await api.profiles.applicationState();'), 1);
  assert.equal(await cold.eval('return await api.browserDiagnostics.increment();'), 1);
  assert.equal(await cold.eval("return localStorage.getItem('isolation');"), 'personal');
  assert.equal((await cold.command('Network.getCookies', { urls: [cookieUrl] })).cookies.find(c => c.name === 'identity')?.value, 'personal');
  const coldLaunchMs = Date.now() - launched;
  await cold.eval('native.window.close(); return true;').catch(error => {
    if (!String(error).includes('target closed')) throw error;
  });
  await until(() => restarted.exitCode !== null, 'Cold application did not shut down');
  assert.equal(restarted.exitCode, 0);
  console.log(`PASS profiles: lazy loading, isolated backends/storage/events/windows, stable window, stale handles, persistence and cold process restart (${coldLaunchMs} ms fixture launch)`);
} finally {
  for (const connection of connections) connection.close();
  if (restarted && restarted.exitCode === null) restarted.kill();
}

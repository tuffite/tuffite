// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
// The Python driver owns application startup, fixtures and OS memory sampling.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const options = JSON.parse(process.env.TUFFITE_MEMORY_OPTIONS);
const targets = await (await fetch(`http://127.0.0.1:${options.port}/json/list`)).json();
const target = targets.find(t => new URL(t.url).pathname.endsWith('/memory.html'));
assert.ok(target, 'Missing benchmark application target');
// Do not attach to Explorer targets: debugging intentionally prevents discard.
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener('open', resolve, { once: true });
  socket.addEventListener('error', reject, { once: true });
});
let sequence = 0;
const pending = new Map();
socket.addEventListener('message', event => {
  const reply = JSON.parse(event.data), request = pending.get(reply.id);
  if (!request) return;
  pending.delete(reply.id);
  clearTimeout(request.timer);
  if (reply.error || reply.result?.exceptionDetails) request.reject(Error(JSON.stringify(reply)));
  else request.resolve(reply.result.result.value);
});
function evaluate(expression) {
  return new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(Error(`Evaluation timed out: ${expression}`));
    }, 20000);
    pending.set(id, { resolve, reject, timer });
    socket.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: {
      expression, awaitPromise: true, returnByValue: true,
    } }));
  });
}
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(expression) {
  for (let i = 0; i < 200; i++) {
    if (await evaluate(expression)) return;
    await sleep(100);
  }
  throw Error(`State deadline exceeded: ${expression}`);
}
const states = () => evaluate('Promise.all(views.map(e => e.getNavigationState()))');
const results = { variant: options.variant, count: options.count, round: options.round, stages: {} };
async function sample(name, settle = 2000) {
  await sleep(settle);
  const documents = await states();
  const rendererIds = [...new Set(documents.map(s => s.rendererProcess.id))];
  const sample = spawnSync(options.python, [options.driver, '--sample', String(options.pid),
    '--renderer-ids', ...rendererIds.map(String)], {
    encoding: 'utf8', timeout: 20000,
  });
  assert.equal(sample.status, 0, sample.stderr);
  results.stages[name] = { ...JSON.parse(sample.stdout), documents };
  if (options.progressFile) writeFileSync(options.progressFile, JSON.stringify(results, null, 2));
}
try {
  await until("document.readyState === 'complete'");
  await evaluate('window.views = []');
  await sample('empty');
  await evaluate(`(() => {
    const host = document.createElement('div');
    host.style.cssText = 'position:fixed;inset:0;display:grid;grid-template-columns:repeat(5,160px);grid-auto-rows:120px;background:white';
    document.body.replaceChildren(host);
    for (let i = 0; i < ${options.count}; i++) {
      const e = document.createElement('explorer');
      e.policy = 'navigation: browser; freeze: never; discard: never';
      e.style.cssText = 'display:block;width:160px;height:120px';
      e.src = ${JSON.stringify(options.origin)} + '/page/' + i;
      views.push(e); host.append(e);
    }
  })()`);
  await until("(async()=> (await Promise.all(views.map(e=>e.getNavigationState()))).every(s=>!s.isLoading && s.title==='Memory ready'))()");
  const initial = await states();
  const rendererIds = new Set(initial.map(s => s.rendererProcess.id));
  assert.equal(rendererIds.size, options.variant === 'per-instance' ? options.count : 1,
    'Unexpected Explorer process sharing');
  results.explorerRendererCount = rendererIds.size;
  await sample('visible');
  assert.equal(results.stages.visible.explorerRendererPids.length, rendererIds.size,
    'Native renderer IDs must map to real OS processes');
  await evaluate("views.forEach(e => e.style.display = 'none')");
  await sample('hidden_running');
  if (options.variant === 'shared') {
    await evaluate("views.forEach(e => e.policy = 'navigation: browser; freeze: 0ms; discard: never')");
    await until('(async()=> (await Promise.all(views.map(e=>e.getNavigationState()))).every(s=>s.frozen))()');
    const frozenAt = performance.now();
    await sample('frozen');
    for (const seconds of options.observeFreezeSeconds ?? []) {
      await sample(`frozen_${seconds}s`, Math.max(0, seconds * 1000 - (performance.now() - frozenAt)));
    }
    const partial = Math.floor(options.count / 2);
    const showSurvivor = partial > 0 && options.visibleSurvivorSeconds > 0;
    if (partial) {
      assert.deepEqual(await evaluate(`Promise.all(views.slice(0,${partial}).map(e=>e.discard()))`),
        Array(partial).fill(true));
      await until(`(async()=> (await Promise.all(views.slice(0,${partial}).map(e=>e.getNavigationState()))).every(s=>s.discarded))()`);
      await sample('half_discarded', 4000);
      if (showSurvivor) {
        await evaluate("views.at(-1).style.display = 'block'");
        await until('(async()=> !(await views.at(-1).getNavigationState()).frozen)()');
        await sample('half_discarded_active', options.visibleSurvivorSeconds * 1000);
      }
    }
    const end = showSurvivor ? options.count - 1 : options.count;
    assert.deepEqual(await evaluate(`Promise.all(views.slice(${partial},${end}).map(e=>e.discard()))`),
      Array(end - partial).fill(true));
    if (showSurvivor) {
      await evaluate("views.at(-1).style.display = 'none'");
      await until('(async()=> (await views.at(-1).getNavigationState()).discarded || await views.at(-1).discard())()');
    }
    await until('(async()=> (await Promise.all(views.map(e=>e.getNavigationState()))).every(s=>s.discarded))()');
    await sample('all_discarded', 4000);
    assert.deepEqual(results.stages.all_discarded.explorerRendererPids, [],
      'Discarding the last page must release the actual renderer process');
    const remainingPids = new Set(results.stages.all_discarded.processes.map(p => p.pid));
    assert.ok(results.stages.visible.explorerRendererPids.every(pid => !remainingPids.has(pid)),
      'The originally loaded Explorer renderer must exit after final discard');
    const start = performance.now();
    await evaluate("views.forEach(e => e.style.display = 'block')");
    await until("(async()=> (await Promise.all(views.map(e=>e.getNavigationState()))).every(s=>!s.isLoading && !s.discarded && s.title==='Memory ready'))()")
      .catch(async error => { throw Error(`${error.message}: ${JSON.stringify(await states())}`); });
    results.restoreAllMs = performance.now() - start;
    const restored = await states();
    assert.ok(restored.every((s, i) => s.documentId !== initial[i].documentId));
    assert.equal(new Set(restored.map(s => s.rendererProcess.id)).size, 1);
    await sample('restored');
    assert.equal(results.stages.restored.explorerRendererPids.length, 1);
  }
  console.log(JSON.stringify(results));
} finally {
  socket.close();
}

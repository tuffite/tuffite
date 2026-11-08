// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
import assert from 'node:assert/strict';
import process from 'node:process';
import { createRequire } from 'node:module';
import { promisify } from 'node:util';
import { execFile } from 'node:child_process';
const require = createRequire(new URL('../ui/package.json', import.meta.url));
const { chromium } = require('playwright-core');
const browser = await chromium.connectOverCDP(
  `http://127.0.0.1:${process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257}`,
);
const path = '/ui/dist/fixtures/configuration.html';
const allowed = 'playground://main' + path;
const denied = 'playground://public' + path;
const otherScheme = 'playground-resource://main' + path;
try {
  const page = browser
    .contexts()
    .flatMap((context) => context.pages())
    .find((page) => page.url() === allowed);
  assert.ok(page, 'Missing configuration fixture');
  const probe = () => page.evaluate(() => globalThis.configurationAtFirstScript);
  assert.deepEqual(await probe(), { shellApi: true, explorer: true, legacyInvoke: 'undefined' });
  const directories = () => page.evaluate(async () => {
    const {path}=await import('/tuffite.js');
    const result=await path.directories();
    return typeof result.resource==='string' && typeof result.data==='string';
  });
  assert.equal(await directories(),true,'Typed Mojo must reach the native service');
  assert.equal(await page.evaluate(()=>{
    const {handle0,handle1}=Mojo.createMessagePipe();
    try { Mojo.bindInterface('tuffite.wire.Path',handle1,'process');return false; }
    catch(error) { return error.name==='NotAllowedError'; }
    finally {handle0.close();}
  }),true,'A document broker must forbid the process interface scope');
  assert.equal(await page.evaluate(()=>new Promise(resolve=>{
    const {handle0,handle1}=Mojo.createMessagePipe();
    const timer=setTimeout(()=>{handle0.close();resolve(false);},5000);
    const watcher=handle0.watch({peerClosed:true},()=>{
      clearTimeout(timer);watcher.cancel();handle0.close();resolve(true);
    });
    Mojo.bindInterface('blink.mojom.PermissionService',handle1);
  })),true,'The document broker must reject unrelated Chromium interfaces');
  assert.deepEqual(
    await page.evaluate(() => {
      const frame = document.createElement('iframe');
      document.body.append(frame);
      const result = {
        shellApi: typeof frame.contentWindow.Mojo?.bindInterface === 'function',
        explorer:
          frame.contentDocument.createElement('explorer').constructor.name ===
          'HTMLExplorerElement',
      };
      frame.remove();
      return result;
    }),
    { shellApi: false, explorer: true },
    'Initial same-origin child must inherit Explorer without main-frame-only Shell API',
  );
  await page.evaluate(() => {
    globalThis.sandboxConfiguration = null;
    addEventListener('message', (event) => {
      if (event.data?.configurationProbe) globalThis.sandboxConfiguration = event.data.value;
    });
    const frame = document.createElement('iframe');
    frame.sandbox = 'allow-scripts';
    frame.srcdoc = `<script>parent.postMessage({configurationProbe:true,value:{shellApi:typeof globalThis.Mojo?.bindInterface==='function',explorer:document.createElement('explorer').constructor.name==='HTMLExplorerElement'}},'*')</script>`;
    document.body.append(frame);
  });
  await page.waitForFunction(() => globalThis.sandboxConfiguration !== null);
  assert.deepEqual(await page.evaluate(() => globalThis.sandboxConfiguration), {
    shellApi: false,
    explorer: false,
  });
  await page.goto(denied);
  assert.deepEqual(await probe(), { shellApi: false, explorer: false, legacyInvoke: 'undefined' });
  await page.goBack();
  assert.deepEqual(await probe(), { shellApi: true, explorer: true, legacyInvoke: 'undefined' });
  assert.equal(await directories(),true,'Back navigation must retain or restore the document broker');
  await page.goto(otherScheme);
  assert.deepEqual(
    await probe(),
    { shellApi: false, explorer: false, legacyInvoke: 'undefined' },
    'Scheme registration must not imply native API permission',
  );
  await page.goto(allowed);
  assert.deepEqual(await probe(), { shellApi: true, explorer: true, legacyInvoke: 'undefined' });
  if (process.platform !== 'win32') {
    const { stdout } = await promisify(execFile)('ps', ['-ww', '-axo', 'pid=,ppid=,args=']);
    const rows = stdout
      .split('\n')
      .map((line) => line.trim().match(/^(\d+)\s+(\d+)\s+(.*)$/))
      .filter(Boolean);
    const descendants = new Set([Number(process.env.TUFFITE_PLAYGROUND_PID)]);
    for (let i = 0; i < rows.length; ++i)
      for (const row of rows) if (descendants.has(Number(row[2]))) descendants.add(Number(row[1]));
    const children = rows.filter(
      (row) => descendants.has(Number(row[1])) && /--type=/.test(row[3]),
    );
    assert.ok(children.length, 'Missing Chromium children');
    for (const child of children) {
      assert.doesNotMatch(
        child[3],
        /--tuffite-(?:app-schemes|shell-api-origins|explorer-origins|dev-token|dev-url)(?:=|\s)/,
        `Unexpected configuration payload in child ${child[1]}`,
      );
    }
  }
  console.log(
    'PASS configuration: first script, initial blank child, opaque sandbox, navigation/back, business schemes and child arguments',
  );
} finally {
  await browser.close();
}

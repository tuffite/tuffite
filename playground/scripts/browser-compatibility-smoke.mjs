// Local API regression; opt in to a separate real-service acceptance check.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { readTurnstileResult } from '../ui/src/browser-compatibility-probe.js';
const require = createRequire(new URL('../ui/package.json', import.meta.url));
const { chromium } = require('playwright-core');
const probe = await readFile(new URL('../ui/src/browser-compatibility-probe.js', import.meta.url), 'utf8');
const worker = await readFile(new URL('../ui/src/browser-compatibility-worker.js', import.meta.url), 'utf8');
const navigationHeaders = [];
const server = createServer((request, response) => {
  if (request.url.startsWith('/turnstile-')) {
    const pass = request.url === '/turnstile-pass';
    response.setHeader('Content-Type', 'text/html');
    response.end(`<!doctype html><title>Turnstile test-key fixture</title><body data-turnstile-test="pending"><h1>Test-key integration only</h1>
      <script>window.fixturePass=()=>document.body.dataset.turnstileTest='pass';window.fixtureFail=()=>document.body.dataset.turnstileTest='fail';</script>
      <div class="cf-turnstile" data-sitekey="${pass ? '1x00000000000000000000AA' : '2x00000000000000000000AB'}" data-response-field="false" data-callback="fixturePass" data-error-callback="fixtureFail"></div>
      <script src="https://challenges.cloudflare.com/turnstile/v0/api.js" async defer></script></body>`);
  } else if (request.url.endsWith('.js')) {
    response.setHeader('Content-Type', 'text/javascript');
    response.end(request.url === '/browser-compatibility-worker.js' ? worker : probe);
  } else {
    navigationHeaders.push({
      userAgent: request.headers['user-agent'],
      language: request.headers['accept-language'],
      encoding: request.headers['accept-encoding'],
      brands: request.headers['sec-ch-ua'],
    });
    response.setHeader('Content-Type', 'text/html');
    response.end(`<!doctype html><title>Standard origin compatibility</title><body><h1>Browser API fixture</h1><script type="module">import {runBrowserCompatibilityProbe} from '/probe.js';document.body.dataset.compatibilityReport=JSON.stringify(await runBrowserCompatibilityProbe());</script></body>`);
  }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const fixture = `http://127.0.0.1:${server.address().port}/`;
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9245);
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
let root;
let controlId;
try {
  root = browser.contexts().flatMap(context => context.pages()).find(page => /\/(index|compatibility)\.html/.test(page.url()));
  assert.ok(root, 'Playground main page is required');
  await root.goto(new URL('compatibility.html', root.url()).href);
  const info = await root.evaluate(async url => (await import('/tuffite.js')).window.create({label:'compatibility-control',url,title:'Standard origin compatibility',bounds:{x:120,y:120,width:900,height:800},focused:false}), fixture);
  controlId = info.id;
  const windowDeadline = Date.now() + 15000;
  let reference;
  while (!reference) {
    reference = browser.contexts().flatMap(context => context.pages()).find(page => page.url() === fixture);
    assert.ok(Date.now() < windowDeadline, 'Standard-origin window was not created');
    if (!reference) await new Promise(resolve => setTimeout(resolve, 100));
  }
  await reference.waitForFunction(() => !!document.body.dataset.compatibilityReport);
  const baseline = JSON.parse(await reference.evaluate(() => document.body.dataset.compatibilityReport));
  const lab = new URL('compatibility.html', root.url()); lab.searchParams.set('fixture', fixture);
  await root.goto(lab.href);
  const deadline = Date.now() + 15000;
  while ((await root.locator('explorer').evaluate(element => element.getNavigationState())).url !== fixture) {
    assert.ok(Date.now() < deadline, 'Explorer fixture did not commit');
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  await root.locator('#compatibility-local').click();
  await root.waitForFunction(() => document.querySelector('#compatibility-report').textContent.startsWith('{'), null, {timeout: 30000});
  const text = await root.locator('#compatibility-report').innerText();
  assert.ok(text.startsWith('{'), text);
  const report = JSON.parse(text);
  report.application = report.parent;
  report.parent = baseline;
  assert.equal(baseline.environment.parentIsSelf, true, 'Standalone reference window must have no embedding parent');
  assert.equal(baseline.environment.topIsSelf, true);
  assert.equal(report.explorer.environment.parentIsSelf, false, 'Ordinary browser pages must see their Explorer embedding parent');
  assert.equal(report.explorer.environment.topIsSelf, false);
  assert.equal(report.explorer.environment.parentIsTop, true);
  assert.equal(report.application.environment.origin, await root.evaluate(() => location.origin), 'Application security origin must match its standard asset URL origin');
  assert.equal(report.application.checks.worker.ok, true, `Application-scheme Worker failed: ${JSON.stringify(report.application.checks.worker)}`);
  console.log(JSON.stringify({local: report.parent.ok && report.explorer.ok ? 'passed' : 'failed', checks: Object.keys(report.explorer.checks), engine: report.explorer.checks.clientHints.value.fullVersionList, applicationSchemeWorker: report.application.checks.worker}, null, 2));
  if (process.env.TUFFITE_TURNSTILE_TEST_KEYS === '1') {
    for (const expected of ['pass', 'fail']) {
      await root.locator('explorer').evaluate((element, url) => { element.src = url; }, fixture + 'turnstile-' + expected);
      const end = Date.now() + 30000;
      let actual;
      do {
        await new Promise(resolve => setTimeout(resolve, 200));
        actual = await root.locator('explorer').evaluate(async element => {
          const state = await element.getNavigationState();
          return element.executeJavaScript("document.body.dataset.turnstileTest || 'pending'", {expectedDocumentId: state.documentId});
        }).catch(() => 'pending');
        if (actual === expected) break;
      } while (Date.now() < end);
      assert.equal(actual, expected, `Official Turnstile test-key callback: ${expected}`);
      console.log(`turnstile-test-key-${expected}=passed (integration only)`);
    }
  }
  if (process.env.TUFFITE_TURNSTILE_LIVE === '1') {
    await root.locator('#compatibility-turnstile').click();
    const end = Date.now() + 90000;
    let result;
    do {
      await new Promise(resolve => setTimeout(resolve, 2000));
      // The troubleshooter scrolls to its diagnostic warning automatically.
      // Keep the challenge visible, as it is during ordinary user verification.
      try { result = await root.locator('explorer').evaluate((element, expression) => element.executeJavaScript(expression), `scrollTo(0, 0); (${readTurnstileResult.toString()})()`); } catch {}
      if (result?.status === 'success') break;
    } while (Date.now() < end);
    console.log(JSON.stringify({turnstile: result || {status: 'pending'}}));
    await root.locator('#compatibility-result').click();
    assert.equal(result?.status, 'success', 'Real Turnstile acceptance did not succeed; local tests do not prove acceptance');
  } else console.log('turnstile=not-tested (set TUFFITE_TURNSTILE_LIVE=1 to require real acceptance)');
  for (const role of ['parent', 'explorer']) {
    for (const [name, check] of Object.entries(report[role].checks)) assert.equal(check.ok, true, `${role}.${name}: ${check.error}`);
    assert.equal(report[role].environment.webdriver, false);
  }
  assert.deepEqual(report.explorer.environment.languages, report.parent.environment.languages);
  assert.deepEqual(report.explorer.checks.clientHints.value, report.parent.checks.clientHints.value);
  assert.equal(report.explorer.checks.webgl.value.renderer, report.parent.checks.webgl.value.renderer);
  assert.ok(navigationHeaders.length >= 2, 'Both standard-origin navigations must reach the server');
  for (const headers of navigationHeaders) {
    assert.equal(headers.userAgent, baseline.environment.userAgent);
    assert.equal(headers.language, 'en-US,en;q=0.9');
    assert.match(headers.brands, /Chromium/);
    for (const encoding of ['gzip', 'deflate', 'br', 'zstd']) assert.ok(headers.encoding.split(', ').includes(encoding), `Missing compression: ${encoding}`);
  }
  console.log('navigation-headers=passed');
} finally {
  if (root && controlId !== undefined) await root.evaluate(async id => (await import('/tuffite.js')).window.close(id), controlId).catch(() => {});
  await browser.close(); await new Promise(resolve => server.close(resolve));
}

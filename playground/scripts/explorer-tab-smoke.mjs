import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
const require = createRequire(new URL('../ui/package.json', import.meta.url));
const {chromium} = require('playwright-core');
const port = Number(process.env.TUFFITE_PLAYGROUND_CDP_PORT || 9257);
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
const pages = () => browser.contexts().flatMap(context => context.pages());
const app = pages().find(page => page.url().includes('/index.html'));
assert.ok(app, 'Playground main window is required');
const base = new URL('./', app.url()).href;
const server = createServer((request, response) => {
  response.setHeader('Content-Type', 'text/html');
  response.end('<!doctype html><title>Cross-site tab fixture</title><p>needle after cross-site navigation</p>');
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const fixture = `http://127.0.0.1:${server.address().port}/`;
const wait = async (check, message) => {
  const end = Date.now() + 15000;
  do { if (await check()) return; await new Promise(resolve => setTimeout(resolve, 50)); } while (Date.now() < end);
  throw new Error(message);
};
const state = id => app.locator(id).evaluate(element => element.getNavigationState());
const execute = (id, expression) => app.locator(id).evaluate((element, expression) => element.executeJavaScript(expression), expression);
const find = async (text, options = {}) => {
  const id = await app.locator('#tab-primary').evaluate((element, {text, options}) => element.findInPage(text, options), {text, options});
  let result;
  await wait(async () => {
    result = await app.evaluate(id => window.tabDemo.events.findLast(event => event.type === 'findresult' && event.detail.requestId === id && event.detail.finalUpdate)?.detail, id);
    return !!result;
  }, 'Native find result did not finish');
  return result;
};
try {
  await app.goto(new URL('tabs.html', base).href);
  await wait(async () => (await state('#tab-primary')).isLoading === false && (await state('#tab-control')).url.endsWith('/tab-sample.html'), 'Tab fixtures did not load');
  const primary = '#tab-primary'; const control = '#tab-control';
  const initial = await state(primary);
  assert.equal(initial.zoomFactor, 1); assert.equal(initial.audioMuted, false); assert.equal(initial.crashed, false);
  await assert.rejects(() => app.evaluate(() => navigator.keyboard.lock(['KeyA'])));
  console.log('keyboard-lock-policy=passed (unsupported top-level lock rejects)');
  // Search the real frame tree rather than using window.find()/DOM text scans.
  assert.equal((await find('needle')).matches, 4, 'Find must include the child iframe');
  const next = await find('needle', {findNext: true});
  assert.equal(next.matches, 4); assert.ok(next.activeMatchOrdinal > 1);
  const previous = await find('needle', {findNext: true, forward: false});
  assert.notEqual(previous.activeMatchOrdinal, next.activeMatchOrdinal);
  assert.equal((await find('needle', {matchCase: true})).matches, 3);
  assert.equal((await find('text-that-is-not-present')).matches, 0);
  await assert.rejects(() => app.locator(primary).evaluate(element => element.findInPage('')));
  assert.equal(await app.locator(primary).evaluate(element => element.stopFindInPage('invalid')), false);
  assert.equal(await app.locator(primary).evaluate(element => element.stopFindInPage('keep')), true);
  assert.equal(await app.locator(primary).evaluate(element => element.stopFindInPage('activate')), true);
  assert.equal(await app.locator(primary).evaluate(element => element.stopFindInPage()), true);
  console.log('find=passed (iframe, case, next/previous, no-match, invalid, stop)');

  const baseline = await execute(primary, '({ratio:devicePixelRatio,width:innerWidth})');
  const parentRatio = await app.evaluate(() => devicePixelRatio);
  assert.equal(await app.locator(primary).evaluate(element => element.setZoomFactor(1.5)), true);
  await wait(async () => Math.abs((await execute(primary, 'devicePixelRatio')) / baseline.ratio - 1.5) < 0.01, 'Native page zoom did not affect rendering');
  assert.equal(await app.locator(primary).evaluate(element => element.getZoomFactor()), 1.5);
  assert.equal((await state(control)).zoomFactor, 1);
  assert.equal(await execute(control, 'devicePixelRatio'), baseline.ratio);
  assert.equal(await app.evaluate(() => devicePixelRatio), parentRatio);
  for (const factor of [0, 0.24, 5.01, -1]) assert.equal(await app.locator(primary).evaluate((element, factor) => element.setZoomFactor(factor), factor), false);
  await assert.rejects(() => app.locator(primary).evaluate(element => element.setZoomFactor(NaN)));
  assert.ok((await execute(primary, 'innerWidth')) < baseline.width);
  const zoomEvents = await app.evaluate(() => window.tabDemo.events.filter(event => event.type === 'zoomchange'));
  assert.ok(zoomEvents.some(event => event.detail.zoomFactor === 1.5));

  assert.equal(await app.locator(primary).evaluate(element => element.setAudioMuted(true)), true);
  await wait(async () => (await state(primary)).audioMuted, 'Native muting state did not update');
  await wait(async () => await app.evaluate(() => window.tabDemo.events.some(event => event.role === 'primary' && event.type === 'audiostatechange' && event.detail.audioMuted)), 'Native mute event missing');
  assert.equal((await state(control)).audioMuted, false);
  await wait(async () => (await state(primary)).favicons.length > 0, 'Native favicon candidates missing');
  assert.ok((await state(primary)).favicons.some(icon => icon.url.includes('/tab-fixture-icon')));
  const dimensions = await execute(primary, '({width:outerWidth,height:outerHeight,x:screenX,y:screenY})');
  assert.deepEqual(dimensions, await app.evaluate(() => ({width:outerWidth,height:outerHeight,x:screenX,y:screenY})));
  console.log('zoom/audio/favicon/window=passed (native rendering and sibling isolation)');

  // Native tab state belongs to the WebContents and survives transfer.
  const transferred = await app.evaluate(async () => {
    const first = document.querySelector('#tab-primary');
    const before = await first.getNavigationState();
    const transfer = await first.detach();
    if (await first.setZoomFactor(2) || await first.setAudioMuted(false)) throw new Error('Detached element accepted a control');
    const receiver = document.createElement('explorer');
    receiver.navigation = 'allow';
    receiver.style.cssText = first.style.cssText;
    first.replaceWith(receiver); receiver.id = 'tab-primary';
    await receiver.attach(transfer);
    return {before, after: await receiver.getNavigationState()};
  });
  assert.equal(transferred.before.documentId, transferred.after.documentId);
  assert.equal(transferred.after.zoomFactor, 1.5); assert.equal(transferred.after.audioMuted, true);
  assert.deepEqual(transferred.after.favicons, transferred.before.favicons);
  // Install fresh listeners: the old element no longer owns this page.
  await app.locator(primary).evaluate(element => {
    for (const type of ['renderprocessgone', 'findresult', 'audiostatechange', 'zoomchange']) element.addEventListener(type, event => window.tabDemo.events.push({type, detail:event.detail}));
  });
  await app.locator(primary).evaluate((element, url) => { element.src = url; }, fixture);
  await wait(async () => (await state(primary)).url === fixture && !(await state(primary)).isLoading, 'Cross-site navigation did not finish');
  assert.equal((await state(primary)).zoomFactor, 1.5);
  assert.equal((await state(primary)).audioMuted, true);
  assert.equal((await find('needle')).matches, 1);
  assert.equal((await state(control)).zoomFactor, 1);
  console.log('transfer/cross-site=passed');

  // A guaranteed connection refusal provides a deterministic native error document.
  const errorTarget = pages().find(page => page.url() === fixture);
  assert.ok(errorTarget, 'Fixture debugging target missing');
  const refused = createServer(); await new Promise(resolve => refused.listen(0, '127.0.0.1', resolve));
  const refusedURL = `http://127.0.0.1:${refused.address().port}/?private-value-do-not-render`;
  await new Promise(resolve => refused.close(resolve));
  await app.locator(primary).evaluate((element, url) => { element.src = url; }, refusedURL);
  await wait(async () => (await state(primary)).url === refusedURL && !(await state(primary)).isLoading, 'Error navigation did not finish');
  // Error documents have opaque origins: retain the production script denial,
  // and inspect only this fixture's error document through the test debugger.
  await assert.rejects(() => execute(primary, 'document.title'));
  assert.equal(await errorTarget.title(), 'Page could not be loaded');
  const errorText = await errorTarget.locator('body').innerText();
  assert.match(errorText, /CONNECTION_REFUSED/); assert.ok(!errorText.includes('private-value-do-not-render'));
  await app.locator(primary).evaluate((element, url) => { element.src = url; }, fixture);
  await wait(async () => (await state(primary)).url === fixture && !(await state(primary)).isLoading, 'Error recovery did not finish');
  console.log('network-error=passed (native error code, no failed URL disclosure, recovery)');

  // Crash only this isolated fixture renderer; the host and control must survive.
  const target = pages().find(page => page.url() === fixture);
  assert.ok(target);
  const session = await target.context().newCDPSession(target);
  void session.send('Page.crash').catch(() => {});
  await wait(async () => (await state(primary)).crashed, 'Renderer crash was not exposed');
  await wait(async () => await app.evaluate(() => window.tabDemo.events.some(event => event.type === 'renderprocessgone' && event.detail.crashed)), 'Renderer exit event missing');
  assert.equal((await state(control)).crashed, false);
  assert.equal(await app.locator(primary).evaluate(element => element.reload()), true);
  await wait(async () => !(await state(primary)).crashed && await execute(primary, "document.title === 'Cross-site tab fixture'").catch(() => false), 'Reload did not recover the crashed renderer');
  assert.equal((await state(primary)).zoomFactor, 1.5); assert.equal((await state(primary)).audioMuted, true);
  console.log('renderer-recovery=passed (host/sibling survived; zoom/mute retained)');
} finally {
  await browser.close(); await new Promise(resolve => server.close(resolve));
}

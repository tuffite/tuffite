import '/shell_api.js';
import { runBrowserCompatibilityProbe, readTurnstileResult } from './browser-compatibility-probe.js';
const status = document.querySelector('#compatibility-status');
const output = document.querySelector('#compatibility-report');
const explorer = document.createElement('explorer');
explorer.navigation = 'allow';
explorer.style.height = '650px';
document.querySelector('#compatibility-host').append(explorer);
const fixture = new URLSearchParams(location.search).get('fixture');
const fixtureURL = fixture ? new URL(fixture) : null;
if (fixtureURL && !(fixtureURL.protocol === 'http:' && ['127.0.0.1', 'localhost'].includes(fixtureURL.hostname))) throw new Error('Fixtures must use local loopback HTTP');
const sample = fixtureURL?.href || new URL('./compatibility-sample.html', location.href).href;
explorer.src = sample;
async function read(expression) {
  const state = await explorer.getNavigationState();
  return explorer.executeJavaScript(expression, { expectedDocumentId: state.documentId });
}
document.querySelector('#compatibility-local').onclick = async () => {
  status.textContent = 'Running local checks';
  try {
    if ((await explorer.getNavigationState()).url !== sample) {
      throw new Error(`Reload this lab to run local checks in the fixture. Current URL: ${(await explorer.getNavigationState()).url}; expected: ${sample}`);
    }
    const parent = await runBrowserCompatibilityProbe();
    let child;
    const deadline = Date.now() + 15000;
    do {
      child = await read(`JSON.parse(document.body.dataset.compatibilityReport || 'null')`);
      if (child) break;
      await new Promise(resolve => setTimeout(resolve, 100));
    } while (Date.now() < deadline);
    if (!child) throw new Error('Explorer fixture probe did not finish');
    const report = { parent, explorer: child };
    output.textContent = JSON.stringify(report, null, 2);
    status.textContent = child.ok
      ? (parent.ok ? 'Local checks passed; Turnstile not tested' : 'Explorer checks passed; application scheme checks differ; Turnstile not tested')
      : 'Local checks failed';
  } catch (error) { status.textContent = 'Local checks failed'; output.textContent = error.message; }
};
document.querySelector('#compatibility-turnstile').onclick = () => {
  explorer.src = 'https://debug.challenges.cloudflare.com/';
  status.textContent = 'Turnstile pending'; output.textContent = '';
};
document.querySelector('#compatibility-result').onclick = async () => {
  try {
    const state = await explorer.getNavigationState();
    if (state.url !== 'https://debug.challenges.cloudflare.com/') throw new Error('Open the troubleshooter first.');
    // Only read the status. Do not export the report's IP, session ID or tokens.
    const result = await read(`(${readTurnstileResult.toString()})()`);
    output.textContent = JSON.stringify(result, null, 2);
    status.textContent = result.status === 'success' ? 'Turnstile passed' : `Turnstile ${result.status}`;
  } catch (error) { status.textContent = 'Result unavailable'; output.textContent = error.message; }
};

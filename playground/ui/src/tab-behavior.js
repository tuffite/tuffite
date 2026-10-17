import '/shell_api.js';
const hosts = document.querySelector('#tab-hosts');
const status = document.querySelector('#tab-status');
const output = document.querySelector('#tab-output');
const explorers = [];
const events = [];
let loaded = 0;
for (let i = 0; i < 2; ++i) {
  const explorer = document.createElement('explorer');
  explorer.id = i ? 'tab-control' : 'tab-primary';
  explorer.navigation = 'allow'; explorer.style.cssText = 'display:block;width:100%;height:330px;margin:1rem 0';
  for (const type of ['findresult', 'zoomchange', 'audiostatechange', 'faviconchange', 'renderprocessgone']) {
    explorer.addEventListener(type, event => {
      events.push({role: i ? 'control' : 'primary', type, detail: event.detail});
      if (events.length > 100) events.shift();
      output.textContent = JSON.stringify(events, null, 2);
      if (type === 'findresult' && !i) status.textContent = `${event.detail.matches} matches; active ${event.detail.activeMatchOrdinal}`;
      if (type === 'renderprocessgone') status.textContent = 'Renderer exited; reload to recover';
    });
  }
  explorer.addEventListener('loadstop', () => { if (++loaded >= 2) status.textContent = 'Fixtures ready'; });
  explorer.src = new URL('./tab-sample.html', location.href).href;
  hosts.append(explorer); explorers.push(explorer);
}
window.tabDemo = {explorers, events};
const primary = explorers[0];
const run = action => async () => {
  try { const result = await action(); if (result === false) throw new Error('Request was rejected'); }
  catch (error) { status.textContent = error.message; }
};
const find = (findNext, forward = true) => run(() => primary.findInPage(document.querySelector('#tab-query').value, {findNext, forward, matchCase: document.querySelector('#tab-case').checked}));
document.querySelector('#tab-find').onclick = find(false);
document.querySelector('#tab-next').onclick = find(true);
document.querySelector('#tab-previous').onclick = find(true, false);
document.querySelector('#tab-clear').onclick = run(() => primary.stopFindInPage());
document.querySelector('#tab-zoom').onchange = run(() => primary.setZoomFactor(Number(document.querySelector('#tab-zoom').value)));
document.querySelector('#tab-mute').onchange = run(() => primary.setAudioMuted(document.querySelector('#tab-mute').checked));
document.querySelector('#tab-reload').onclick = run(() => primary.reload());
document.querySelector('#tab-state').onclick = run(async () => { output.textContent = JSON.stringify(await primary.getNavigationState(), null, 2); });

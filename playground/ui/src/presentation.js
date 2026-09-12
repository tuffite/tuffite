import './tabs.js';

let theme = 'dark';
try {
  theme = localStorage.getItem('tuffite-theme') ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
} catch { /* Native storage policy may restrict persistence. */ }
if (!['dark', 'light'].includes(theme)) theme = 'dark';
const toggle = document.createElement('button');
toggle.type = 'button';
function applyTheme() {
  document.documentElement.dataset.theme = theme;
  toggle.textContent = theme === 'dark' ? 'Light mode' : 'Dark mode';
  toggle.setAttribute('aria-label', `Switch to ${theme === 'dark' ? 'light' : 'dark'} mode`);
  toggle.title = 'Change Playground color theme';
}
toggle.addEventListener('click', () => {
  theme = theme === 'dark' ? 'light' : 'dark';
  applyTheme();
  try { localStorage.setItem('tuffite-theme', theme); } catch { /* Theme still changes without persistence. */ }
});
applyTheme();
const header = document.querySelector('.page-header');
const actions = document.createElement('div');
if (header) {
  actions.className = 'page-actions';
  const status = header.querySelector('.status-pill');
  if (status) actions.append(status);
  header.append(actions);
} else {
  actions.className = 'lab-actions';
  const back = document.createElement('a');
  back.href = './index.html';
  back.textContent = '← Playground';
  actions.append(back);
  document.querySelector('main')?.prepend(actions);
}
actions.append(toggle);
const descriptions = {
  'explorer-reload-cache': 'Reload the embedded page without using its cached resources',
  'explorer-recreate': 'Destroy this browser surface and create a fresh page',
  'explorer-stress': 'Recreate the surface 20 times to exercise lifecycle cleanup',
  'explorer-state': 'Inspect the current document, navigation, storage, and renderer state',
  'explorer-execute': 'Run a script inside the embedded page’s isolated world',
};
for (const [id, title] of Object.entries(descriptions)) {
  const button = document.getElementById(id);
  if (button) button.title = title;
}

if (!window.tuffite) {
  const status = document.querySelector('#explorer-suite-status');
  if (status) {
    status.textContent = 'Browser preview';
    status.className = 'status-pill';
    status.title = 'Launch with tuff dev to exercise native APIs';
  }
  const actionStatus = document.querySelector('#explorer-action-status');
  if (actionStatus) actionStatus.textContent = 'Visual preview — launch in Tuffite to use native controls.';
  for (const control of document.querySelectorAll('.tab-panel button')) {
    control.disabled = true;
    control.title = 'Available inside the Tuffite native application';
  }
}

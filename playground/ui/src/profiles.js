import { window as nativeWindow } from '/tuffite.js';
import { profiles, browserDiagnostics, commands } from '/shell_api.js';

const status = document.querySelector('#status');
const value = document.querySelector('#saved-value');
let switching = false;
const identity = JSON.parse(await browserDiagnostics.identity());
document.querySelector('#identity').textContent = JSON.stringify(identity, null, 2);
value.value = localStorage.getItem('profile-lab-value') || '';
document.querySelector('#save-value').onclick = () => {
  localStorage.setItem('profile-lab-value', value.value);
  status.textContent = `Saved in ${identity.profile}`;
};
document.querySelector('#increment').onclick = async () => {
  document.querySelector('#counter').textContent = await browserDiagnostics.increment();
};
async function refresh() {
  const items = JSON.parse(await profiles.list());
  const container = document.querySelector('#profiles');
  container.replaceChildren();
  for (const item of items) {
    const button = document.createElement('button');
    button.textContent = `${item.name}${item.loaded ? ' (loaded)' : ''}`;
    button.dataset.profile = item.id;
    button.disabled = item.id === identity.profile;
    button.onclick = () => switchProfile(item.id).catch(showError);
    container.append(button);
    const open = document.createElement('button');
    open.textContent = `Open ${item.name} in a new window`;
    open.onclick = () => openProfileWindow(item.id).catch(showError);
    container.append(open);
  }
  document.querySelector('#create-profile').disabled = items.some(item => item.id === 'work');
}
function showError(error) { status.textContent = String(error); }
async function openProfileWindow(id) {
  const view = await profiles.prepare(id, location.href);
  let windowId;
  try {
    windowId = await profiles.createWindow();
    await profiles.attach(windowId, view);
  } catch (error) {
    if (windowId !== undefined) await profiles.closeWindow(windowId).catch(() => {});
    await profiles.releaseView(view).catch(() => {});
    throw error;
  }
}
async function switchProfile(id) {
  if (switching) return;
  switching = true;
  let prepared;
  try {
    const current = await nativeWindow.current();
    status.textContent = `Preparing ${id}…`;
    prepared = await profiles.prepare(id, location.href);
    const previous = await profiles.attach(BigInt(current.id), prepared);
    prepared = undefined;
    // The old document remains alive until the attach reply is delivered.
    await profiles.releaseView(previous);
  } catch (error) {
    if (prepared) await profiles.releaseView(prepared).catch(() => {});
    throw error;
  } finally { switching = false; }
}
document.querySelector('#create-profile').onclick = async () => {
  try { await profiles.create('work', 'Work'); await refresh(); }
  catch (error) { showError(error); }
};
await refresh();
window.profileLab = { identity, switchProfile, openProfileWindow, refresh, api: { profiles, browserDiagnostics, commands }, nativeWindow };

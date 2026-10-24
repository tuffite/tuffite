import * as frameworkApi from "/tuffite.js";
function describe(info) {
  const state = [
    info.focused && 'focused',
    info.minimized && 'minimized',
    info.maximized && 'maximized',
    info.fullscreen && 'fullscreen',
  ].filter(Boolean).join(', ') || 'normal';
  const {x, y, width, height} = info.bounds;
  return `${info.title}\nlabel: ${info.label || '(dynamic)'}\nid: ${info.id}\nbounds: ${x}, ${y}, ${width} × ${height}\nstate: ${state}`;
}

export function setupWindowManagement({onReady}) {
  const output = document.querySelector('#window');
  let managedId;
  const childUrl = new URL('./window.html', document.baseURI).href;

  async function currentManaged() {
    if (managedId === undefined) {
      const created = await frameworkApi.window.create({
        label: 'managed',
        url: childUrl,
        title: 'Playground managed child',
        bounds: {x: 120, y: 120, width: 560, height: 420},
      });
      managedId = created.id;
    }
    const info = (await frameworkApi.window.list()).find(({id}) => id === managedId);
    if (!info) {
      managedId = undefined;
      return currentManaged();
    }
    return info;
  }

  async function run(action) {
    try {
      output.textContent = describe(await action());
    } catch (error) {
      output.textContent = `Window error: ${error.message}`;
    }
  }

  document.querySelector('#window-create').addEventListener('click', () =>
    run(currentManaged));
  document.querySelector('#window-focus').addEventListener('click', () =>
    run(async () => frameworkApi.window.focus((await currentManaged()).id)));
  document.querySelector('#window-minimize').addEventListener('click', () =>
    run(async () => frameworkApi.window.minimize((await currentManaged()).id)));
  document.querySelector('#window-maximize').addEventListener('click', () =>
    run(async () => frameworkApi.window.maximize((await currentManaged()).id)));
  document.querySelector('#window-restore').addEventListener('click', () =>
    run(async () => frameworkApi.window.restore((await currentManaged()).id)));
  document.querySelector('#window-fullscreen').addEventListener('click', () =>
    run(async () => {
      const info = await currentManaged();
      return frameworkApi.window.setFullscreen(!info.fullscreen, info.id);
    }));
  document.querySelector('#window-list').addEventListener('click', async () => {
    try {
      const windows = await frameworkApi.window.list();
      output.textContent = windows.map(describe).join('\n\n');
    } catch (error) {
      output.textContent = `Window error: ${error.message}`;
    }
  });
  document.querySelector('#window-close').addEventListener('click', async () => {
    try {
      if (managedId !== undefined) await frameworkApi.window.close(managedId);
      managedId = undefined;
      output.textContent = 'Managed child closed.';
    } catch (error) {
      output.textContent = `Window error: ${error.message}`;
    }
  });

  return async function checkWindowManagement() {
    try {
      const current = await frameworkApi.window.current();
      if (current.label !== 'main') {
        throw new Error(`unexpected primary window label: ${current.label}`);
      }
      await frameworkApi.window.setTitle('Playground · Tuffite', current.id);
      const initialBounds = await frameworkApi.window.getBounds(current.id);
      const child = await frameworkApi.window.create({
        label: 'smoke-child',
        url: childUrl,
        title: 'Playground window smoke',
        bounds: {x: 120, y: 120, width: 520, height: 360},
        focused: false,
      });
      await frameworkApi.window.setBounds(
          {x: 140, y: 140, width: 540, height: 380}, child.id);
      await frameworkApi.window.focus(child.id);
      await frameworkApi.window.minimize(child.id);
      await frameworkApi.window.restore(child.id);
      await frameworkApi.window.maximize(child.id);
      await frameworkApi.window.restore(child.id);
      await frameworkApi.window.setFullscreen(true, child.id);
      await frameworkApi.window.setFullscreen(false, child.id);
      const windows = await frameworkApi.window.list();
      if (!windows.some(({id}) => id === current.id) ||
          !windows.some(({id}) => id === child.id)) {
        throw new Error('created windows were not registered');
      }
      await frameworkApi.window.close(child.id);
      output.textContent =
          `Current window ${current.id} (${initialBounds.width} × ${initialBounds.height})\n` +
          'All 12 window operations passed. Use the controls to manage a child.';
      onReady();
    } catch (error) {
      output.textContent = `Window error: ${error.message}`;
    }
  };
}

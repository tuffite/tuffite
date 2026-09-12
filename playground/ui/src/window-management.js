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
      const created = await window.tuffite.window.create({
        label: 'managed',
        url: childUrl,
        title: 'Playground managed child',
        bounds: {x: 120, y: 120, width: 560, height: 420},
      });
      managedId = created.id;
    }
    const info = (await window.tuffite.window.list()).find(({id}) => id === managedId);
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
    run(async () => window.tuffite.window.focus((await currentManaged()).id)));
  document.querySelector('#window-minimize').addEventListener('click', () =>
    run(async () => window.tuffite.window.minimize((await currentManaged()).id)));
  document.querySelector('#window-maximize').addEventListener('click', () =>
    run(async () => window.tuffite.window.maximize((await currentManaged()).id)));
  document.querySelector('#window-restore').addEventListener('click', () =>
    run(async () => window.tuffite.window.restore((await currentManaged()).id)));
  document.querySelector('#window-fullscreen').addEventListener('click', () =>
    run(async () => {
      const info = await currentManaged();
      return window.tuffite.window.setFullscreen(!info.fullscreen, info.id);
    }));
  document.querySelector('#window-list').addEventListener('click', async () => {
    try {
      const windows = await window.tuffite.window.list();
      output.textContent = windows.map(describe).join('\n\n');
    } catch (error) {
      output.textContent = `Window error: ${error.message}`;
    }
  });
  document.querySelector('#window-close').addEventListener('click', async () => {
    try {
      if (managedId !== undefined) await window.tuffite.window.close(managedId);
      managedId = undefined;
      output.textContent = 'Managed child closed.';
    } catch (error) {
      output.textContent = `Window error: ${error.message}`;
    }
  });

  return async function checkWindowManagement() {
    try {
      const current = await window.tuffite.window.current();
      if (current.label !== 'main') {
        throw new Error(`unexpected primary window label: ${current.label}`);
      }
      await window.tuffite.window.setTitle('Playground · Tuffite', current.id);
      const initialBounds = await window.tuffite.window.getBounds(current.id);
      const child = await window.tuffite.window.create({
        label: 'smoke-child',
        url: childUrl,
        title: 'Playground window smoke',
        bounds: {x: 120, y: 120, width: 520, height: 360},
        focused: false,
      });
      await window.tuffite.window.setBounds(
          {x: 140, y: 140, width: 540, height: 380}, child.id);
      await window.tuffite.window.focus(child.id);
      await window.tuffite.window.minimize(child.id);
      await window.tuffite.window.restore(child.id);
      await window.tuffite.window.maximize(child.id);
      await window.tuffite.window.restore(child.id);
      await window.tuffite.window.setFullscreen(true, child.id);
      await window.tuffite.window.setFullscreen(false, child.id);
      const windows = await window.tuffite.window.list();
      if (!windows.some(({id}) => id === current.id) ||
          !windows.some(({id}) => id === child.id)) {
        throw new Error('created windows were not registered');
      }
      await window.tuffite.window.close(child.id);
      output.textContent =
          `Current window ${current.id} (${initialBounds.width} × ${initialBounds.height})\n` +
          'All 12 window operations passed. Use the controls to manage a child.';
      onReady();
    } catch (error) {
      output.textContent = `Window error: ${error.message}`;
    }
  };
}

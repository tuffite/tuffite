import '/tuffite-shell-api.js';

const output = document.querySelector('#window-info');
let current;

function show(info) {
  const {x, y, width, height} = info.bounds;
  output.textContent =
      `Opaque WindowId: ${info.id}\nBounds: ${x}, ${y}, ${width} × ${height}`;
}

try {
  current = await window.tuffite.window.current();
  show(current);
} catch (error) {
  output.textContent = `Window error: ${error.message}`;
}

document.querySelector('#child-title').addEventListener('click', async () => {
  current = await window.tuffite.window.setTitle(
      `Managed child · ${new Date().toLocaleTimeString()}`, current.id);
  show(current);
});

document.querySelector('#child-resize').addEventListener('click', async () => {
  const bounds = await window.tuffite.window.getBounds(current.id);
  current = await window.tuffite.window.setBounds({
    x: bounds.x + 12,
    y: bounds.y + 12,
    width: bounds.width === 640 ? 560 : 640,
    height: bounds.height === 460 ? 420 : 460,
  }, current.id);
  show(current);
});

document.querySelector('#child-close').addEventListener('click', () =>
  window.tuffite.window.close(current.id));

// Runs in the document being tested, including an ordinary HTTPS Explorer.
// Return API results only; never read authentication data or existing storage.
export async function runBrowserCompatibilityProbe() {
  const report = { checks: {}, environment: {} };
  async function check(name, action) {
    try { report.checks[name] = { ok: true, value: await action() }; }
    catch (error) { report.checks[name] = { ok: false, error: error.name + ': ' + error.message }; }
  }
  const timeout = (promise) => Promise.race([
    promise, new Promise((_, reject) => setTimeout(() => reject(new Error('Probe timed out')), 5000)),
  ]);
  report.environment = {
    secureContext: isSecureContext, webdriver: navigator.webdriver,
    origin: self.origin,
    parentIsSelf: parent === window, topIsSelf: top === window,
    parentIsTop: parent === top,
    userAgent: navigator.userAgent, languages: [...navigator.languages],
    focused: document.hasFocus(), visibility: document.visibilityState,
    viewport: [innerWidth, innerHeight], window: [outerWidth, outerHeight],
  };
  await check('clientHints', async () => navigator.userAgentData
    ? navigator.userAgentData.getHighEntropyValues(['architecture', 'bitness', 'platformVersion', 'fullVersionList'])
    : null);
  await check('legacyNavigationTiming', () => {
    if (typeof window.chrome?.csi !== 'function' || typeof window.chrome?.loadTimes !== 'function') throw new Error('Legacy navigation timing API unavailable');
    const csi = window.chrome.csi(); const times = window.chrome.loadTimes();
    if (Math.abs(csi.startE - performance.timeOrigin) > 2 || Math.abs(times.requestTime * 1000 - performance.timeOrigin) > 2) throw new Error('Legacy timing and Performance time origin disagree');
    if (!Number.isFinite(csi.pageT) || csi.pageT < 0 || Math.abs(csi.pageT - (Date.now() - csi.startE)) > 100) throw new Error('Legacy elapsed time does not describe this navigation');
    return { csi, times };
  });
  await check('canvas', () => {
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = 2;
    const context = canvas.getContext('2d');
    context.fillStyle = '#123456'; context.fillRect(0, 0, 2, 2);
    const pixels = [...context.getImageData(0, 0, 1, 1).data];
    if (pixels.join() !== '18,52,86,255') throw new Error('Canvas readback differs from the drawn pixel');
    if (canvas.toDataURL() !== canvas.toDataURL()) throw new Error('Canvas encoding is unstable');
    return { pixels, stable: true };
  });
  for (const type of ['webgl', 'webgl2']) await check(type, () => {
    const canvas = document.createElement('canvas'); canvas.width = canvas.height = 2;
    const gl = canvas.getContext(type);
    if (!gl) throw new Error('Context unavailable');
    gl.clearColor(1, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);
    const pixel = new Uint8Array(4); gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
    if (pixel.join() !== '255,0,0,255' || gl.getError() !== gl.NO_ERROR) throw new Error('WebGL readback failed');
    const debug = gl.getExtension('WEBGL_debug_renderer_info');
    const result = { pixel: [...pixel], vendor: debug && gl.getParameter(debug.UNMASKED_VENDOR_WEBGL), renderer: debug && gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) };
    gl.getExtension('WEBGL_lose_context')?.loseContext();
    return result;
  });
  await check('crypto', async () => {
    const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode('abc')));
    const hex = [...digest].map(byte => byte.toString(16).padStart(2, '0')).join('');
    if (hex !== 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad') throw new Error('SHA-256 mismatch');
    return true;
  });
  await check('worker', async () => {
    const worker = new Worker(new URL('./browser-compatibility-worker.js', import.meta.url), {type: 'module'});
    try {
      const result = await timeout(new Promise((resolve, reject) => {
        worker.onmessage = event => resolve(event.data); worker.onerror = event => reject(new Error(event.message || 'Worker failed')); worker.postMessage(null);
      }));
      if (JSON.stringify(result.languages) !== JSON.stringify(navigator.languages) || result.userAgent !== navigator.userAgent) throw new Error('Document and Worker metadata disagree');
      return result;
    } finally { worker.terminate(); }
  });
  const key = 'tuffite-compatibility-' + crypto.randomUUID();
  await check('localStorage', () => {
    try { localStorage.setItem(key, 'fixture'); if (localStorage.getItem(key) !== 'fixture') throw new Error('Readback failed'); return true; }
    finally { localStorage.removeItem(key); }
  });
  await check('indexedDB', async () => {
    const db = await timeout(new Promise((resolve, reject) => {
      const request = indexedDB.open(key, 1);
      request.onupgradeneeded = () => request.result.createObjectStore('fixture');
      request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error);
    }));
    try {
      await timeout(new Promise((resolve, reject) => {
        const transaction = db.transaction('fixture', 'readwrite'); transaction.objectStore('fixture').put('value', 'key');
        transaction.oncomplete = resolve; transaction.onabort = () => reject(transaction.error);
      }));
      return await timeout(new Promise((resolve, reject) => {
        const request = db.transaction('fixture').objectStore('fixture').get('key');
        request.onsuccess = () => request.result === 'value' ? resolve(true) : reject(new Error('Readback failed'));
        request.onerror = () => reject(request.error);
      }));
    } finally { db.close(); indexedDB.deleteDatabase(key); }
  });
  await check('cacheStorage', async () => {
    try {
      // CacheStorage accepts only HTTP(S) request keys, even in a secure app scheme.
      // This is a synthetic cache entry; it does not issue a network request.
      const cache = await caches.open(key); const url = 'https://tuffite.invalid/compatibility-fixture';
      await cache.put(url, new Response('fixture'));
      if (await (await cache.match(url)).text() !== 'fixture') throw new Error('Readback failed');
      return true;
    } finally { await caches.delete(key); }
  });
  await check('permissions', async () => {
    const states = {};
    for (const name of ['notifications', 'geolocation', 'camera', 'microphone', 'clipboard-read', 'clipboard-write']) {
      try { states[name] = (await navigator.permissions.query({name})).state; }
      catch (error) { states[name] = error.name; }
    }
    return states;
  });
  report.ok = Object.values(report.checks).every(check => check.ok);
  return report;
}

export function readTurnstileResult() {
  const text = document.querySelector('#technical-details')?.textContent || '';
  const match = text.match(/Turnstile Status:\s*([a-z_]+)/i);
  return {
    status: match?.[1]?.toLowerCase() || 'pending',
    errorCode: document.querySelector('#turnstile-section')?.textContent.match(/Error:\s*(\d+)/)?.[1] || null,
    issue: document.querySelector('#issue-title')?.textContent?.trim() || null,
  };
}

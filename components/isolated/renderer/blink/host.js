((native, load) => {
  'use strict';
  const worlds = new Map();
  const views = new Map();
  let nextView = 0;
  const invalid = () => new DOMException('Isolated handle is disposed', 'InvalidStateError');
  const abort = () => new DOMException('Operation aborted', 'AbortError');
  function encode(value) {
    const seen = new Set();
    function check(item, depth = 0) {
      if (depth > 64) throw new DOMException('JSON nesting exceeds 64', 'DataCloneError');
      if (item === null || typeof item === 'string' || typeof item === 'boolean') return;
      if (typeof item === 'number' && Number.isFinite(item)) return;
      if (typeof item !== 'object' || seen.has(item) ||
          (!Array.isArray(item) && Object.getPrototypeOf(item) !== Object.prototype &&
           Object.getPrototypeOf(item) !== null)) {
        throw new DOMException('Only JSON values can cross worlds', 'DataCloneError');
      }
      seen.add(item);
      for (const key of Object.keys(item)) check(item[key], depth + 1);
      seen.delete(item);
    }
    check(value);
    const text = JSON.stringify(value);
    if (text.length > 256 * 1024) throw new DOMException('JSON exceeds 256 Ki characters', 'DataCloneError');
    return text;
  }
  function properties(value) {
    if (value === null || typeof value !== 'object' || Array.isArray(value))
      throw new TypeError('properties must be a JSON object');
    for (const key of Object.keys(value)) {
      if (!/^[A-Za-z][A-Za-z0-9_]*$/.test(key) ||
          ['constructor', 'prototype', 'innerHTML', 'outerHTML', 'srcdoc'].includes(key) ||
          key.startsWith('on')) throw new TypeError(`Invalid property: ${key}`);
    }
    return encode(value);
  }
  native('sink', (worldId, action, viewId, requestId, method, json) => {
    const world = worlds.get(worldId);
    if (!world) return;
    if (action === 'error') {
      world.target.dispatchEvent(new ErrorEvent('error', {message: method}));
      return;
    }
    const view = views.get(viewId);
    if (!view || view.worldId !== worldId || view.disposed) return;
    if (action === 'cancel') {
      view.pending.get(requestId)?.abort();
      view.pending.delete(requestId);
      return;
    }
    if (view.pending.size >= 128 || view.pending.has(requestId)) {
      native('complete', worldId, requestId, false, 'Too many requests');
      return;
    }
    const controller = new AbortController();
    view.pending.set(requestId, controller);
    // Invoke the host handler after leaving the extension's native stack.
    Promise.resolve().then(() => {
      if (controller.signal.aborted) throw abort();
      if (!view.onRequest) throw new DOMException('No request handler', 'NotSupportedError');
      return view.onRequest(Object.freeze({method, params: JSON.parse(json), signal: controller.signal}));
    }).then(result => finish(true, encode(result ?? null)), error => {
      finish(false, String(error?.message ?? error).slice(0, 4096));
    }).catch(error => finish(false, String(error?.message ?? error).slice(0, 4096)));
    function finish(ok, result) {
      if (view.pending.get(requestId) !== controller) return;
      view.pending.delete(requestId);
      if (!view.disposed && worlds.has(worldId)) {
        try { native('complete', worldId, requestId, ok, result); } catch { /* closed */ }
      }
    }
  });
  function closeView(view, callNative = true) {
    if (view.disposed) return;
    view.disposed = true;
    views.delete(view.id);
    for (const pending of view.pending.values()) pending.abort();
    view.pending.clear();
    if (callNative) native('dispose', view.worldId, view.id);
  }
  const api = Object.freeze({
    async createWorld(options) {
      const name = options?.name;
      if (typeof name !== 'string' || !name || name.length > 128)
        throw new TypeError('A registered world name is required');
      const source = await load(name);
      const id = native('create', name, source);
      const target = new EventTarget();
      const state = {target, disposed: false};
      worlds.set(id, state);
      Object.defineProperties(target, {
        name: {enumerable: true, value: name},
        mount: {value: async (container, options) => {
          if (state.disposed) throw invalid();
          if (!(container instanceof HTMLElement) || container.ownerDocument !== document)
            throw new TypeError('container must belong to the current document');
          if (typeof options?.tag !== 'string' || !options.tag.includes('-'))
            throw new TypeError('A custom element tag is required');
          if (options.onRequest !== undefined && typeof options.onRequest !== 'function')
            throw new TypeError('onRequest must be a function');
          const json = properties(options.properties ?? {});
          const viewId = ++nextView;
          const view = {id: viewId, worldId: id, onRequest: options.onRequest,
                        pending: new Map(), disposed: false};
          views.set(viewId, view);
          try { native('mount', id, container, options.tag, json, viewId); }
          catch (error) { closeView(view, false); throw error; }
          return Object.freeze({
            async update(value) {
              if (view.disposed || state.disposed) throw invalid();
              native('update', id, viewId, properties(value));
            },
            async dispose() { closeView(view); },
          });
        }},
        dispose: {value: async () => {
          if (state.disposed) return;
          state.disposed = true;
          // Revoke host capabilities before running extension cleanup.
          for (const view of [...views.values()]) {
            if (view.worldId === id) closeView(view, false);
          }
          try { native('disposeWorld', id); }
          finally { worlds.delete(id); }
        }},
      });
      return Object.freeze(target);
    },
  });
  const root = globalThis.tuffite ?? new EventTarget();
  Object.defineProperty(root, 'isolated', {enumerable: true, value: api});
  if (!globalThis.tuffite) Object.defineProperty(globalThis, 'tuffite', {value: root});
})

((name, registry, native) => {
  'use strict';
  const lifetime = new AbortController();
  const views = new Map();
  const contexts = new WeakMap();
  const pending = new Map();
  let nextRequest = 0;
  let disposed = false;
  const abort = () => new DOMException('Operation aborted', 'AbortError');
  const invalid = () => new DOMException('Isolated context is disposed', 'InvalidStateError');
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
  function close(id) {
    const view = views.get(id);
    if (!view) return;
    views.delete(id);
    view.closed = true;
    view.controller.abort();
    view.node.remove();
  }
  function makeContext(id, view) {
    return Object.freeze({
      signal: view.controller.signal,
      async request(method, params = null, options = {}) {
        if (disposed || view.closed) throw invalid();
        if (options.signal?.aborted) throw abort();
        if (typeof method !== 'string' || !method || method.length > 128)
          throw new TypeError('Invalid method');
        if (pending.size >= 128) throw new DOMException('Too many requests', 'QuotaExceededError');
        const json = encode(params);
        const requestId = ++nextRequest;
        return new Promise((resolve, reject) => {
          const cancel = () => {
            if (!pending.has(requestId)) return;
            pending.delete(requestId);
            cleanup();
            try { native('cancel', id, requestId); } catch { /* world closed */ }
            reject(abort());
          };
          const cleanup = () => {
            options.signal?.removeEventListener('abort', cancel);
            view.controller.signal.removeEventListener('abort', cancel);
          };
          pending.set(requestId, {resolve, reject, cleanup});
          options.signal?.addEventListener('abort', cancel, {once: true});
          view.controller.signal.addEventListener('abort', cancel, {once: true});
          try { native('request', id, requestId, method, json); }
          catch (error) { pending.delete(requestId); cleanup(); reject(error); }
        });
      },
    });
  }
  const api = Object.freeze({
    name,
    customElements: registry,
    signal: lifetime.signal,
    context(element) {
      if (disposed) throw invalid();
      if (!(element instanceof HTMLElement)) throw new TypeError('An HTMLElement is required');
      for (let node = element; node; node = node.parentNode ?? node.host) {
        const record = contexts.get(node);
        if (record) {
          if (record.closed) throw invalid();
          return record.context;
        }
      }
      throw new DOMException('Element does not belong to this world', 'SecurityError');
    },
  });
  const root = new EventTarget();
  Object.defineProperty(root, 'isolated', {enumerable: true, value: api});
  Object.defineProperty(globalThis, 'tuffite', {value: root});
  addEventListener('error', event => {
    if (!disposed) native('error', 0, 0, String(event.message).slice(0, 4096));
  }, {signal: lifetime.signal});
  return Object.freeze({
    mount(container, tag, json, id) {
      if (disposed) throw invalid();
      if (!registry.get(tag)) throw new TypeError(`Unregistered component: ${tag}`);
      const node = document.createElement('div');
      node.dataset.isolatedWorld = name;
      const shadow = node.attachShadow({mode: 'closed', customElementRegistry: registry});
      const element = document.createElement(tag, {customElementRegistry: registry});
      if (!(element instanceof registry.get(tag))) throw new Error('Component construction failed');
      const view = {node, element, controller: new AbortController(), closed: false};
      view.context = makeContext(id, view);
      contexts.set(element, view);
      contexts.set(shadow, view);
      views.set(id, view);
      try {
        Object.assign(element, JSON.parse(json));
        shadow.append(element);
        container.append(node);
      } catch (error) { close(id); throw error; }
    },
    update(id, json) {
      const view = views.get(id);
      if (disposed || !view) throw invalid();
      Object.assign(view.element, JSON.parse(json));
    },
    dispose(id) { close(id); },
    disposeWorld() {
      if (disposed) return;
      disposed = true;
      for (const id of [...views.keys()]) close(id);
      lifetime.abort();
    },
    complete(id, ok, json) {
      const request = pending.get(id);
      if (!request) return;
      pending.delete(id);
      request.cleanup();
      if (ok) request.resolve(JSON.parse(json));
      else request.reject(new Error(json));
    },
  });
})

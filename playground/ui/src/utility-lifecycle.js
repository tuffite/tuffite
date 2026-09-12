const encoder = new TextEncoder();
const decoder = new TextDecoder();

const OPERATION = Object.freeze({
  status: 0,
  echo: 1,
  crash: 2,
  delay: 3,
  cancellable: 4,
  concurrent: 5,
});
let nextRequestId = 1n;

function request(operation, message = '') {
  const body = encoder.encode(message);
  const payload = new Uint8Array(body.byteLength + 1);
  payload[0] = operation;
  payload.set(body, 1);
  return payload;
}

function readU64(view, offset) {
  return view.getBigUint64(offset, true);
}

function decodeSnapshot(bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.byteLength < 28) {
    throw new Error('utility returned a malformed lifecycle snapshot');
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return {
    processId: view.getUint32(0, true),
    instanceId: readU64(view, 4),
    invocationCount: readU64(view, 12),
    uptimeMs: readU64(view, 20),
    message: decoder.decode(bytes.subarray(28)),
  };
}

function formatSnapshot(snapshot) {
  return [
    `process: ${snapshot.processId}`,
    `instance: ${snapshot.instanceId}`,
    `invocations: ${snapshot.invocationCount}`,
    `uptime: ${snapshot.uptimeMs} ms`,
    `message: ${snapshot.message || '(empty)'}`,
  ].join('\n');
}

async function invoke(operation, message = '') {
  const requestId = nextRequestId++;
  return decodeSnapshot(await window.tuffite.utility.invoke(
      'echo', requestId, request(operation, message)));
}

async function checkCancellation() {
  const requestId = nextRequestId++;
  const invocation = window.tuffite.utility.invoke(
      'echo', requestId, request(OPERATION.cancellable));
  await new Promise((resolve) => setTimeout(resolve, 30));
  await window.tuffite.utility.cancel('echo', requestId);
  try {
    await invocation;
    throw new Error('cancelled utility invocation unexpectedly succeeded');
  } catch (error) {
    if (error.message === 'cancelled utility invocation unexpectedly succeeded') throw error;
  }
}

export function setupUtilityLifecycle({onReady, onConcurrentReady}) {
  const output = document.querySelector('#utility');
  const message = document.querySelector('#utility-message');
  const status = document.querySelector('#utility-status');
  const echo = document.querySelector('#utility-echo');
  const crash = document.querySelector('#utility-crash');

  async function show(operation, payload = '') {
    output.textContent = 'Invoking supervised utility service…';
    const snapshot = await invoke(operation, payload);
    output.textContent = formatSnapshot(snapshot);
    return snapshot;
  }

  status.addEventListener('click', async () => {
    try {
      await show(OPERATION.status);
    } catch (error) {
      output.textContent = `Utility error: ${error.message}`;
    }
  });

  echo.addEventListener('click', async () => {
    try {
      await show(OPERATION.echo, message.value);
    } catch (error) {
      output.textContent = `Utility error: ${error.message}`;
    }
  });

  crash.addEventListener('click', async () => {
    output.textContent =
        'Crashing the utility child intentionally; this request is not replayed…';
    try {
      await invoke(OPERATION.crash);
      output.textContent = 'Unexpectedly survived the crash request.';
    } catch (error) {
      output.textContent =
          `Expected child exit: ${error.message}\n` +
          'Use “Status / restart” to observe a new process and instance.';
    }
  });

  return async function checkUtilityLifecycle() {
    try {
      const first = await show(OPERATION.echo, 'hello from sandbox');
      const second = await invoke(OPERATION.echo, 'hello again');
      if (first.processId !== second.processId ||
          first.instanceId !== second.instanceId ||
          second.invocationCount !== first.invocationCount + 1n) {
        throw new Error('utility instance was not reused');
      }
      output.textContent = formatSnapshot(second) +
          '\n\nInstance reuse verified. Testing async completion…';
      const started = performance.now();
      let rendererTimerFired = false;
      setTimeout(() => { rendererTimerFired = true; }, 0);
      const delayed = await invoke(OPERATION.delay);
      if (performance.now() - started < 100 || delayed.message !== 'async-complete' ||
          !rendererTimerFired) {
        throw new Error('utility async invocation did not complete asynchronously');
      }
      const concurrentStarted = performance.now();
      const [concurrentA, concurrentB] = await Promise.all([
        invoke(OPERATION.concurrent),
        invoke(OPERATION.concurrent),
      ]);
      const concurrentElapsed = performance.now() - concurrentStarted;
      if (concurrentA.message !== 'concurrent-complete' ||
          concurrentB.message !== 'concurrent-complete' ||
          concurrentA.processId !== concurrentB.processId ||
          concurrentA.instanceId !== concurrentB.instanceId) {
        throw new Error(
            'utility concurrent calls did not meet in the same service instance');
      }
      onConcurrentReady();
      await checkCancellation();
      output.textContent = formatSnapshot(delayed) +
          `\n\nConcurrent unary calls completed in ${concurrentElapsed.toFixed(1)} ms. ` +
          'Async completion and cancellation verified.';
      onReady();
    } catch (error) {
      output.textContent = `Utility error: ${error.message}`;
    }
  };
}

const output = document.querySelector('#result');
const checks = {};
globalThis.__isolatedWaits = {started: 0, aborted: 0};
globalThis.__hostSecret = 'host-only';
function check(name, condition) {
  if (!condition) throw new Error(`Failed: ${name}`);
  checks[name] = true;
}
try {
  check('nativeRegistrationOnly', typeof tuffite.isolated.register === 'undefined');
  const dynamic = await tuffite.isolated.createWorld({name: 'dynamicProbe'});
  await dynamic.dispose();
  check('dynamicPackage', true);
  const first = await tuffite.isolated.createWorld({name: 'weather'});
  const second = await tuffite.isolated.createWorld({name: 'weatherOther'});
  const views = await Promise.all([first, second].map((world, index) => world.mount(
    document.querySelector(index ? '#second' : '#first'), {
      tag: 'weather-card', properties: {data: {title: 'Weather', value: index + 1}},
      onRequest: async ({method, params, signal}) => {
        if (method === 'wait') {
          ++globalThis.__isolatedWaits.started;
          return await new Promise(resolve => signal.addEventListener('abort', () => {
            ++globalThis.__isolatedWaits.aborted;
            resolve(null);
          }, {once: true}));
        }
        if (method !== 'echo') throw new Error('Unknown method');
        return {value: params.value * 10};
      },
    })));
  check('mounted', document.querySelectorAll('[data-isolated-world]').length === 2);
  check('globalsIsolated', globalThis.__worldSecret === undefined);
  check('registryIsolated', customElements.get('weather-card') === undefined);
  await views[0].update({data: {title: 'Updated', value: 7}, streaming: true});
  check('updated', true);
  let invalidProperties = false;
  try { await views[0].update({onload: 'no'}); } catch { invalidProperties = true; }
  check('invalidPropertiesRejected', invalidProperties);
  let denied = false;
  try { await tuffite.isolated.createWorld({name: 'not-registered'}); } catch { denied = true; }
  check('unregisteredDenied', denied);
  let removedDenied = false;
  try { await tuffite.isolated.createWorld({name: 'removed'}); } catch { removedDenied = true; }
  check('removedDenied', removedDenied);
  let failures = 0;
  for (let i = 0; i < 40; ++i) {
    try { await tuffite.isolated.createWorld({name: 'broken'}); }
    catch (error) { if (error.message.includes('ESM fixture failure')) ++failures; }
  }
  check('failedModulesReleaseWorldsAndLeases', failures === 40);
  globalThis.__isolatedLab = {first, second, views};
  document.querySelector('#update').onclick = () => views[0].update({data: {title: 'Interactive update', value: Date.now()}});
  document.querySelector('#dispose').onclick = async () => {
    await first.dispose(); await second.dispose();
    output.textContent = 'Disposed';
  };
  globalThis.__isolatedSmoke = checks;
  output.textContent = JSON.stringify(checks, null, 2);
} catch (error) {
  globalThis.__isolatedSmoke = {error: error.stack};
  output.textContent = error.stack;
}

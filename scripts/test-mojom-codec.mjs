import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {tuffite} from '../packages/tuffite/vite.js';
import {encodeMojom, decodeMojom, createMojomCommandApi} from '../crates/tuffite/src/build/mojom/runtime.mjs';
const source = await readFile(new URL('../playground/generated/index.js', import.meta.url), 'utf8');
const runtimeUrl = new URL('../crates/tuffite/src/build/mojom/runtime.mjs', import.meta.url).href;
const bindings = await import('data:text/javascript,' + encodeURIComponent(
  source.replaceAll("'/tuffite.js'", JSON.stringify(runtimeUrl))));
const {mojomTypes} = bindings;

const hex = JSON.parse(await readFile(new URL('../components/dispatch/test/mojom_sample.inc', import.meta.url), 'utf8'));
const fixture = Uint8Array.from(Buffer.from(hex, 'hex'));
const sample = {
  small:7, large:(1n << 63n) + 17n, enabled:true, visible:false,
  medium:513, signedValue:-12345, fraction:1.25,
  nested:{label:'你好', bytes:new Uint8Array([0,1,255])},
};
assert.deepEqual(encodeMojom(mojomTypes.MojomSample, sample), fixture);
assert.deepEqual(decodeMojom(mojomTypes.MojomSample, fixture), sample);
for (let length = 0; length < fixture.length; length++) {
  assert.throws(() => decodeMojom(mojomTypes.MojomSample, fixture.subarray(0,length)));
}
for (const offset of [4,32,48,64,68,72]) {
  const corrupt = fixture.slice(); corrupt.fill(255,offset,offset + 4);
  assert.throws(() => decodeMojom(mojomTypes.MojomSample, corrupt));
}
assert.throws(() => encodeMojom(mojomTypes.MojomSample, {...sample, small:256}));
assert.throws(() => encodeMojom(mojomTypes.MojomSample, {...sample, large:1n << 64n}));
assert.throws(() => createMojomCommandApi([['__proto__.run','1','1',{},{}]],'demo'));
// Import the actual generated module with its runtime URL resolved for Node.
// It must load without a browser, a native bridge, or application globals.
assert.equal(Object.hasOwn(globalThis, 'playground'), false);
assert.equal(typeof bindings.commands.echoMojom, 'function');
assert.doesNotMatch(source, /Object\.defineProperty\(window|globalThis\.playground/);
let calls = 0;
const api = bindings.createPlaygroundApi(async (channel, id, fingerprint, payload) => {
  calls++;
  assert.equal(channel, 'playground.Commands.EchoMojom');
  assert.equal(typeof id, 'bigint');
  assert.equal(typeof fingerprint, 'bigint');
  const request = decodeMojom(mojomTypes.CommandsEchoMojomRequest, payload);
  assert.deepEqual(request.value, sample);
  return encodeMojom(mojomTypes.CommandsEchoMojomResponse, {result:request.value});
});
assert.ok(Object.isFrozen(api.commands));
assert.deepEqual(await api.commands.echoMojom(sample), sample);
assert.equal(calls, 1);
await assert.rejects(api.commands.echoMojom({...sample, small:256}));
assert.equal(calls, 1, 'invalid input must not reach the transport');
const failing = bindings.createPlaygroundApi(async () => { throw new Error('transport failed'); });
await assert.rejects(failing.commands.echoMojom(sample), /transport failed/);
assert.throws(() => bindings.createPlaygroundApi(null));
// The application ships one self-contained generated module.
const plugin = tuffite({
  root:fileURLToPath(new URL('../playground', import.meta.url)),
  shellApi:{url:'/shell_api.js',file:'generated/index.js'},
});
const facadeId = plugin.resolveId('/shell_api.js');
assert.match(await plugin.load(facadeId), /export const mojomMethods/);
assert.equal(plugin.resolveId('./mojom.js', facadeId), undefined);
assert.equal(plugin.resolveId('../unmanaged.js', facadeId), undefined);
const assets = [];
await plugin.generateBundle.call({emitFile(asset) { assets.push(asset); }});
assert.ok(assets.some(asset => asset.fileName === 'shell_api.js'));
assert.deepEqual(assets.map(asset => asset.fileName), ['shell_api.js']);
console.log('Mojom JS: official C++ fixture, typed values, truncation, corruption and range checks passed');

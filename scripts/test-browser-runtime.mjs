import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';

globalThis.window = globalThis;
const source = await readFile(
    new URL('../packages/tuffite/runtime.js', import.meta.url), 'utf8');
const runtime = await import(
    `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);

function responseFor(request, payload) {
  const bytes = new Uint8Array(20 + payload.byteLength);
  bytes.set(new TextEncoder().encode('CERM'));
  const output = new DataView(bytes.buffer);
  output.setUint16(4, 2, true);
  output.setBigUint64(8, new DataView(
      request.buffer, request.byteOffset).getBigUint64(8, true), true);
  output.setUint32(16, payload.byteLength, true);
  bytes.set(payload, 20);
  return bytes;
}

function encodedString(value) {
  const text = new TextEncoder().encode(value);
  const bytes = new Uint8Array(4 + text.byteLength);
  new DataView(bytes.buffer).setUint32(0, text.byteLength, true);
  bytes.set(text, 4);
  return bytes;
}

let handler;
globalThis.tuffite = {
  invoke: async (channel, body) => handler(channel, new Uint8Array(body)),
};

const api = runtime.createTypedCommandApi([[
  'commands.echo',
  '0x1',
  '0x11',
  [{kind: 'string'}],
  {kind: 'string'},
]], [['commands.protocolVersion', 1]], 'example');
assert.strictEqual(api.commands, api.commands);
assert.strictEqual(api.commands.echo, api.commands.echo);
assert.equal(api.commands.protocolVersion, 1);
handler = (channel, request) => {
  assert.equal(channel, 'example.commands.echo');
  const requestView = new DataView(request.buffer, request.byteOffset);
  assert.equal(requestView.getBigUint64(24, true), 0x11n);
  assert.equal(requestView.getUint32(32, true), 9);
  assert.equal(requestView.getUint32(36, true), 5);
  assert.equal(new TextDecoder().decode(request.subarray(40)), 'hello');
  return responseFor(request, encodedString('world'));
};
assert.equal(await api.commands.echo('hello'), 'world');
await assert.rejects(api.commands.echo(7), /expected string/);

const optional = runtime.createTypedCommandApi([[
  'read',
  '0x2',
  '0x22',
  [],
  {kind: 'optional', value: {kind: 'boolean'}},
]], [], 'example');
handler = (_channel, request) => responseFor(request, Uint8Array.of(2));
await assert.rejects(optional.read(), /invalid optional tag/);

const integerEnum = runtime.createTypedCommandApi([[
  'roundTrip',
  '0x3',
  '0x33',
  [{kind: 'integer'}],
  {kind: 'integer'},
]], [], 'example');
handler = (_channel, request) => responseFor(request, request.subarray(36));
assert.equal(await integerEnum.roundTrip(42), 42);
await assert.rejects(integerEnum.roundTrip(1.5), /safe integer/);

let frameworkRequest;
handler = async (channel, body) => {
  frameworkRequest = {channel, body};
  const bytes = new Uint8Array(14);
  const view = new DataView(bytes.buffer);
  view.setFloat64(0, 204, true);
  view.setUint32(8, 2, true);
  bytes.set([1, 2], 12);
  return bytes;
};
const framework = runtime.createFrameworkApi([[
  'net.request',
  [{kind: 'string'}],
  {kind: 'object', fields: [
    ['status', {kind: 'number'}],
    ['body', {kind: 'bytes'}],
  ]},
]]);
const network = await framework.net.request('https://example.com/');
assert.equal(frameworkRequest.channel, 'tuffite.net.request');
assert.deepEqual(new TextDecoder().decode(frameworkRequest.body.subarray(4)),
    'https://example.com/');
assert.equal(network.status, 204);
assert.deepEqual([...network.body], [1, 2]);
await assert.rejects(framework.net.request(42), /expected string/);

const windowFramework = runtime.createFrameworkApi([[
  'window.create',
  [{kind: 'optional', value: {kind: 'object', fields: [
    ['url', {kind: 'optional', value: {kind: 'string'}}],
    ['focused', {kind: 'optional', value: {kind: 'boolean'}}],
  ]}}],
  {kind: 'bigint'},
]]);
handler = async (channel, body) => {
  assert.equal(channel, 'tuffite.window.create');
  assert.deepEqual([...body.subarray(0, 2)], [1, 1]);
  const urlLength = new DataView(body.buffer, body.byteOffset).getUint32(2, true);
  const urlEnd = 6 + urlLength;
  assert.equal(new TextDecoder().decode(body.subarray(6, urlEnd)),
      'playground://main/window.html');
  assert.deepEqual([...body.subarray(urlEnd)], [1, 0]);
  const response = new Uint8Array(8);
  new DataView(response.buffer).setBigInt64(0, 17n, true);
  return response;
};
assert.equal(await windowFramework.window.create({
  url: 'playground://main/window.html', focused: false,
}), 17n);

// Window methods retain their native identity and bypass the Shell data codec.
let posted;
globalThis.tuffite = {
  invoke: async (channel, body) => handler(channel, new Uint8Array(body)),
  postWindowMessage: (...args) => { posted = args; },
};
const references = runtime.createFrameworkApi([[
  'window.current', [], {kind: 'object', fields: [['id', {kind: 'bigint'}]]},
]]);
handler = async () => {
  const response = new Uint8Array(8);
  new DataView(response.buffer).setBigInt64(0, 17n, true);
  return response;
};
const reference = await references.window.current();
reference.id = 99n;
const message = {value: 42n};
const options = {targetOrigin: 'https://example.com', transfer: []};
assert.equal(reference.postMessage(message, options), undefined);
assert.equal(posted[0], 17n);
assert.equal(posted[1], message);
assert.equal(posted[2], options);
assert.equal('transfer' in reference, false);
assert.deepEqual(Object.keys(reference), ['id']);
assert.throws(() => structuredClone(reference, {transfer: [reference]}));

handler = (_channel, request) => {
  const response = responseFor(request, new TextEncoder().encode('denied'));
  new DataView(response.buffer).setUint16(6, 4, true);
  return response;
};
await assert.rejects(api.commands.echo('hello'), (error) => {
  assert(error instanceof runtime.TuffiteError);
  assert.equal(error.code, 'invalid_argument');
  assert.equal(error.message, 'denied');
  assert.equal(typeof error.requestId, 'bigint');
  return true;
});

globalThis.tuffite.invoke = async () => {
  const error = new Error('transport stopped');
  error.status = 4;
  throw error;
};
await assert.rejects(api.commands.echo('hello'), (error) => {
  assert(error instanceof runtime.TuffiteError);
  assert.equal(error.code, 'unavailable');
  return true;
});

console.log('browser runtime codec tests passed');

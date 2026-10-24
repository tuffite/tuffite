import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';

globalThis.window = globalThis;
const source = await readFile(
    new URL('../crates/tuffite/src/build/mojom/runtime.mjs', import.meta.url), 'utf8');
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

let handler;
globalThis.tuffite = {invoke:async (channel,body)=>handler(channel,new Uint8Array(body))};
const textType = {size:16,fields:[['value',8,0,'string']]};
const api = runtime.createMojomCommandApi([['commands.echo','0x1','0x11',textType,textType,'example.Commands.Echo']],'example');
assert.strictEqual(api.commands,api.commands);
assert.strictEqual(api.commands.echo,api.commands.echo);
assert.equal(Object.isFrozen(api.commands),true);
handler = (channel,request) => {
  assert.equal(channel,'example.Commands.Echo');
  assert.equal(new DataView(request.buffer).getBigUint64(24,true),0x11n);
  assert.deepEqual(runtime.decodeMojom(textType,request.subarray(36)),{value:'hello'});
  return responseFor(request,runtime.encodeMojom(textType,{value:'world'}));
};
assert.equal(await api.commands.echo('hello'),'world');
await assert.rejects(api.commands.echo(7),/invalid Mojom string/);
await assert.rejects(api.commands.echo(),/argument count/);
handler = (_channel,request) => responseFor(request,new Uint8Array(8));
await assert.rejects(api.commands.echo('hello'),/Mojom/);
handler = (_channel,request) => { const response=responseFor(request,runtime.encodeMojom(textType,{value:'wrong'})); new DataView(response.buffer).setBigUint64(8,9999n,true); return response; };
await assert.rejects(api.commands.echo('hello'),error=>error.code==='protocol_mismatch');
const {mojomTypes,mojomMethods} = await import('../packages/tuffite/generated/index.js');
const framework = runtime.createMojomCommandApi(mojomMethods,'tuffite',runtime.invokeFrameworkMojom);
handler = async (channel,body) => {
  assert.equal(channel,'tuffite.Net.Request');
  assert.deepEqual(runtime.decodeMojom(mojomTypes.NetRequestRequest,body),{url:'https://example.com/'});
  return runtime.encodeMojom(mojomTypes.NetRequestResponse,{result:{status:204,body:new Uint8Array([1,2])}});
};
assert.deepEqual(await framework.net.request('https://example.com/'),{status:204,body:new Uint8Array([1,2])});
await assert.rejects(framework.net.request(42),/invalid Mojom string/);
let posted;
globalThis.tuffite.postWindowMessage = (...args) => { posted=args; };
handler = async channel => {
  assert.equal(channel,'tuffite.Window.Current');
  return runtime.encodeMojom(mojomTypes.WindowCurrentResponse,{result:{id:17n,label:'main',title:'test',url:'https://example.com/',
    bounds:{x:0,y:0,width:800,height:600},focused:true,minimized:false,maximized:false,fullscreen:false}});
};
const reference = await framework.window.current();
reference.id=99n;
const message={value:42n};
const options={targetOrigin:'https://example.com',transfer:[]};
reference.postMessage(message,options);
assert.deepEqual(posted,[17n,message,options]);
assert.equal('transfer' in reference,false);

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

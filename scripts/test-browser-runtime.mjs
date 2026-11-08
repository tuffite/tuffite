import assert from 'node:assert/strict';
import * as runtime from '../crates/tuffite/src/build/mojom/runtime.mjs';
import {installMojo,replyTo} from './mojo-test-pipe.mjs';
import {mojomTypes,mojomMethods,createTuffiteApi} from '../packages/tuffite/generated/index.js';
const text={size:16,fields:[['value',8,0,'string']]};
const entries=[['commands.echo',7,text,text,'example.wire.Commands']];
let status=0, sent=0;
installMojo((name,buffer,handle)=>{
  sent++;
  const header=new DataView(buffer);
  assert.equal(name,'example.wire.Commands');
  assert.equal(header.getUint32(12,true),7);
  assert.equal(header.getUint32(16,true),1);
  const params=new Uint8Array(buffer,header.getUint32(0,true));
  assert.deepEqual(runtime.decodeMojom(text,params),{value:'hello'});
  const response={size:32,fields:[['status',8,0,'i32'],['value',16,0,{nullable:text}],['error',24,0,{nullable:'string'}]]};
  handle.writeMessage(replyTo(buffer,runtime.encodeMojom(response,{status,value:status?null:{value:'world'},error:status?'native failure detail':null})),[]);
});
const api=runtime.createMojomCommandApi(entries,'example');
assert.equal(await api.commands.echo('hello'),'world');
assert.equal(Object.isFrozen(api.commands),true);
assert.equal(globalThis.tuffite?.invoke,undefined);
await assert.rejects(api.commands.echo(7),/invalid Mojom string/);
await assert.rejects(api.commands.echo(),/argument count/);
assert.equal(sent,1,'Invalid typed arguments must send no IPC');
for(const [value,code] of [[1,'invalid_argument'],[2,'access_denied'],[3,'unavailable'],[4,'cancelled'],[5,'timed_out'],[6,'resource_exhausted']]) {
  status=value;
  await assert.rejects(api.commands.echo('hello'),error=>error.code===code && error.cause.status===value && error.message==='native failure detail');
}
// Standard Mojo nullable value flags and nested arrays must match the data IR.
const optional={size:40,fields:[['optional',16,0,{nullable:'u64'},[8,0]],['name',24,0,{nullable:'string'}],['items',32,0,{array:'bytes',width:8,boolean:false}]]};
let expected;
installMojo((name,buffer,handle)=>{
  assert.equal(name,'example.wire.Nullable');
  const header=new DataView(buffer);
  const decoded=runtime.decodeMojom(optional,new Uint8Array(buffer,header.getUint32(0,true)));
  assert.deepEqual(decoded,expected);
  const response={size:32,fields:[['status',8,0,'i32'],['value',16,0,{nullable:optional}],['error',24,0,{nullable:'string'}]]};
  handle.writeMessage(replyTo(buffer,runtime.encodeMojom(response,{status:0,value:expected})),[]);
});
const nullable=runtime.createMojomCommandApi([['nullable.run',0,optional,optional,'example.wire.Nullable']],'example');
for(const scalar of [null,0n,17n]) {
  expected={optional:scalar,name:scalar===null?null:'中文',items:[new Uint8Array([0,255]),new Uint8Array()]};
  // The first result field is returned by a generated unary client.
  assert.equal(await nullable.nullable.run(expected.optional,expected.name,expected.items),scalar);
}
installMojo((_name,_buffer,handle)=>handle.close());
const empty={size:8,fields:[]};
const closed=runtime.createMojomCommandApi([['closed.run',0,empty,empty,'example.wire.Closed']],'example');
await assert.rejects(closed.closed.run(),error=>error instanceof runtime.TuffiteError && error.code==='transport_unavailable');
// Typed transport injection has no binary request envelope or channel invocation.
let posted;
globalThis.tuffite={postWindowMessage:(...args)=>posted=args};
const framework=createTuffiteApi(async (endpoint,value)=>{
  assert.equal(endpoint.interfaceName,'tuffite.wire.Window');
  assert.equal(typeof endpoint.ordinal,'number');
  assert.deepEqual(value,{});
  return {result:{id:17n,label:'main',title:'test',url:'https://example.com/',bounds:{x:0,y:0,width:800,height:600},focused:true,minimized:false,maximized:false,fullscreen:false}};
});
const window=await framework.window.current();
window.id=99n;
window.postMessage({hello:true},{targetOrigin:'https://example.com'});
assert.deepEqual(posted,[17n,{hello:true},{targetOrigin:'https://example.com'}]);
console.log('Typed Mojo: real Chromium message headers, ordinal dispatch, correlation, errors and typed injection passed');

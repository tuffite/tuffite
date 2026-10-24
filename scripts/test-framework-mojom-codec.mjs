import assert from 'node:assert/strict';
import {readFile, writeFile} from 'node:fs/promises';
import {encodeMojom, decodeMojom} from '../crates/tuffite/src/build/mojom/runtime.mjs';
import {mojomTypes, BaseDirectory} from '../packages/tuffite/generated/index.js';

const cases = {
  fs: ['FsReadFileRequest', {base:BaseDirectory.Data, path:'配置.bin'}],
  window: ['WindowCreateRequest', {options:{label:'child', url:null, title:null,
    bounds:{x:10,y:20,width:640,height:480}, focused:false}}],
  remoting: ['RemotingStartRequest', {options:{authKey:'secret',
    iceServers:[{urls:['stun:example.test','turn:example.test'],username:'user',credential:null}],
    maxBitrateKbps:null,targetFramerate:60,screenId:0n}}],
};
for (const [name, [type,value]] of Object.entries(cases)) {
  const bytes = encodeMojom(mojomTypes[type],value);
  assert.deepEqual(decodeMojom(mojomTypes[type],bytes),value);
  const path = new URL(`../components/dispatch/test/framework_${name}.inc`,import.meta.url);
  const hex = Buffer.from(bytes).toString('hex');
  if (process.argv.includes('--write-fixtures')) await writeFile(path,JSON.stringify(hex)+'\n');
  assert.equal(JSON.parse(await readFile(path,'utf8')),hex);
  for (let length=0;length<bytes.length;length++) {
    assert.throws(()=>decodeMojom(mojomTypes[type],bytes.subarray(0,length)));
  }
}
assert.throws(()=>encodeMojom(mojomTypes.FsReadFileRequest,{base:99,path:'bad'}));
assert.throws(()=>encodeMojom(mojomTypes.FsReadFileRequest,{base:'data',path:'bad'}));
assert.equal(BaseDirectory[BaseDirectory.Data],'Data');
const nullable = {options:null};
assert.deepEqual(decodeMojom(mojomTypes.WindowCreateRequest,
  encodeMojom(mojomTypes.WindowCreateRequest,nullable)),nullable);
// Importing the generated clients never installs business methods on the bridge.
globalThis.tuffite = {invoke:async()=>{throw new Error('unexpected IPC');}};
const api = await import('../packages/tuffite/generated/index.js');
assert.equal(typeof api.fs.readFile,'function');
assert.equal(api.BaseDirectory.Data,1);
assert.equal(Object.hasOwn(globalThis.tuffite,'fs'),false);
console.log('Framework Mojom: enum, nullable values/references, nested arrays, ESM clients and fixture checks passed');

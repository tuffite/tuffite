import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {performance} from 'node:perf_hooks';
const {encodeMojom,decodeMojom} = await import(process.argv[2] ? pathToFileURL(process.argv[2]).href : new URL('../crates/tuffite/src/build/mojom/runtime.mjs',import.meta.url).href);
const cases = [
 ['small',{size:24,fields:[['n',8,0,'u32'],['text',16,0,'string']]},{n:123,text:'benchmark'},20000],
 ['scalars-16k',{size:16,fields:[['values',8,0,{array:'u32',width:4,boolean:false}]]},{values:Array.from({length:16384},(_,i)=>i)},100],
 ['bytes-1m',{size:16,fields:[['bytes',8,0,'bytes']]},{bytes:new Uint8Array(1024*1024).fill(173)},50],
 ['strings-1k',{size:16,fields:[['values',8,0,{array:'string',width:8,boolean:false}]]},{values:Array.from({length:1024},(_,i)=>`entry-${i}`)},100],
];
const report={runtime:process.version,cases:{}};
for(const [name,schema,value,count] of cases){
 const bytes=encodeMojom(schema,value);
 assert.deepEqual(decodeMojom(schema,bytes),value);
 for(let i=0;i<Math.min(count,200);i++){decodeMojom(schema,encodeMojom(schema,value));}
 const samples=[];
 for(let round=0;round<5;round++){
  const start=performance.now();for(let i=0;i<count;i++){decodeMojom(schema,encodeMojom(schema,value));}
  samples.push((performance.now()-start)*1000/count);
 }
 samples.sort((a,b)=>a-b);
 report.cases[name]={wireBytes:bytes.length,roundTripMedianUs:Math.round(samples[2]*100)/100};
}
console.log(JSON.stringify(report,null,2));

// Typed clients use Chromium's Mojo message pipes and interface bindings.
import {mojo} from './chromium_mojo.mjs';
const encoder = new TextEncoder();
const codecDecoder = new TextDecoder('utf-8', {fatal: true});
const MAX_COLLECTION_LENGTH = 16 * 1024 * 1024;
// Primitive metadata is shared by all schemas, rather than rebuilt per element.
const mojomScalars = Object.freeze(Object.fromEntries(
  ['u8','i8','u16','i16','u32','i32','u64','i64','f32','f64'].map(type => {
    const bits = Number(type.slice(1));
    const unsigned = type[0] === 'u';
    const suffix = `${bits === 64 && type[0] !== 'f' ? 'Big' : ''}${type[0] === 'f' ? 'Float' : unsigned ? 'Uint' : 'Int'}${bits}`;
    return [type, {get:`get${suffix}`, set:`set${suffix}`,
      min:unsigned ? 0 : -(2**(bits-1)), max:unsigned ? 2**bits-1 : 2**(bits-1)-1}];
  })
));

// Handle-free Mojom version-zero data structs. Offsets come from tuff's Wire IR;
// this runtime neither opens Mojo endpoints nor implements message headers.
export function encodeMojom(schema, value) {
  let buffer = new Uint8Array(1024);
  let used = 0;
  let dataView = new DataView(buffer.buffer);
  const view = () => dataView;
  function allocate(size) {
    const aligned = Math.ceil(size / 8) * 8;
    if (!Number.isSafeInteger(size) || size < 8 || used + aligned > MAX_COLLECTION_LENGTH) {
      throw new RangeError('Mojom payload exceeds budget');
    }
    const at = used;
    used += aligned;
    if (used > buffer.length) {
      const next = new Uint8Array(Math.max(used, Math.min(MAX_COLLECTION_LENGTH, buffer.length * 2)));
      next.set(buffer); buffer = next;
      dataView = new DataView(buffer.buffer);
    }
    return at;
  }
  function writeType(field, item, at, bit = 0) {
    if (field && typeof field === 'object') {
      if ('nullable' in field) { if (item == null) return; return writeType(field.nullable, item, at, bit); }
      if ('enum' in field) {
        if (!field.enum.includes(item)) throw new TypeError('invalid Mojom enum');
        view().setInt32(at, item, true); return;
      }
      if ('array' in field) {
        if (!Array.isArray(item) || item.length > MAX_COLLECTION_LENGTH) throw new TypeError('expected bounded Mojom array');
        const size = 8 + (field.boolean ? Math.ceil(item.length/8) : item.length*field.width);
        const target = allocate(size);
        view().setBigUint64(at, BigInt(target-at), true);
        view().setUint32(target, size, true); view().setUint32(target+4, item.length, true);
        item.forEach((value, index) => writeType(field.array, value, target+8+(field.boolean ? Math.floor(index/8) : index*field.width), field.boolean ? index%8 : 0));
        return;
      }
      const target = allocate(field.size);
      view().setBigUint64(at, BigInt(target-at), true); writeStruct(field, item, target); return;
    }
    if (field === 'string' || field === 'bytes') {
      if (field === 'string' ? typeof item !== 'string' : !(item instanceof Uint8Array)) throw new TypeError(`invalid Mojom ${field}`);
      const bytes = field === 'string' ? encoder.encode(item) : item;
      const target = allocate(8+bytes.length);
      view().setBigUint64(at, BigInt(target-at), true);
      view().setUint32(target, 8+bytes.length, true); view().setUint32(target+4, bytes.length, true);
      buffer.set(bytes,target+8); return;
    }
    if (field === 'bool') {
      if (typeof item !== 'boolean') throw new TypeError('expected boolean');
      buffer[at] |= Number(item)<<bit; return;
    }
    const scalar = mojomScalars[field];
    if (field === 'u64' || field === 'i64') {
      if (typeof item !== 'bigint' || (field === 'u64' ? BigInt.asUintN(64,item) : BigInt.asIntN(64,item)) !== item) throw new TypeError('expected bounded bigint');
      view()[scalar.set](at,item,true);
    } else if (field[0] === 'f') {
      if (typeof item !== 'number') throw new TypeError('expected number');
      view()[scalar.set](at,item,true);
    } else {
      if (!scalar || !Number.isInteger(item) || item < scalar.min || item > scalar.max) throw new TypeError(`expected ${field}`);
      view()[scalar.set](at,item,true);
    }
  }
  function writeStruct(type, object, base) {
    if (!object || typeof object !== 'object' || Array.isArray(object)) throw new TypeError('expected Mojom struct');
    view().setUint32(base,type.size,true);
    for (const [name, offset, bit, field, presence] of type.fields) {
      const item = object[name];
      if (presence) {
        if (item == null) continue;
        buffer[base+presence[0]] |= 1<<presence[1];
        writeType(field.nullable,item,base+offset,bit);
      } else writeType(field,item,base+offset,bit);
    }
  }
  writeStruct(schema, value, allocate(schema.size));
  return buffer.slice(0, used);
}

export function decodeMojom(schema, bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.length > MAX_COLLECTION_LENGTH) throw new TypeError('invalid Mojom bytes');
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let claimed = 0;
  function claim(at, size) {
    const end = at + Math.ceil(size / 8) * 8;
    if (!Number.isSafeInteger(at) || !Number.isSafeInteger(size) || at % 8 || at < claimed || size < 8 || end > bytes.length) throw new TypeError('invalid Mojom allocation');
    claimed = end;
  }
  function pointer(at) {
    const distance = view.getBigUint64(at, true);
    if (distance < 8n || distance > BigInt(bytes.length)) throw new TypeError('invalid Mojom pointer');
    return at + Number(distance);
  }
  function readType(field, at, bit = 0) {
    if (field && typeof field === 'object') {
      if ('nullable' in field) return view.getBigUint64(at,true) === 0n ? null : readType(field.nullable,at,bit);
      if ('enum' in field) {
        const value = view.getInt32(at,true);
        if (!field.enum.includes(value)) throw new TypeError('unknown Mojom enum');
        return value;
      }
      if ('array' in field) {
        const target = pointer(at), size = view.getUint32(target,true), length = view.getUint32(target+4,true);
        if (length > MAX_COLLECTION_LENGTH || size !== 8+(field.boolean ? Math.ceil(length/8) : length*field.width)) throw new TypeError('invalid Mojom array');
        claim(target,size);
        return Array.from({length}, (_,index) => readType(field.array,target+8+(field.boolean ? Math.floor(index/8) : index*field.width),field.boolean ? index%8 : 0));
      }
      return readStruct(field,pointer(at));
    }
    if (field === 'string' || field === 'bytes') {
      const target = pointer(at), size = view.getUint32(target,true), length = view.getUint32(target+4,true);
      if (size !== length+8) throw new TypeError('invalid Mojom array header');
      claim(target,size);
      const data = bytes.subarray(target+8,target+size);
      return field === 'string' ? codecDecoder.decode(data) : data.slice();
    }
    if (field === 'bool') return (view.getUint8(at)&(1<<bit)) !== 0;
    return view[mojomScalars[field].get](at,true);
  }
  function readStruct(type,at) {
    claim(at,type.size);
    if (view.getUint32(at,true) !== type.size || view.getUint32(at+4,true) !== 0) throw new TypeError('unsupported Mojom struct version/size');
    return Object.fromEntries(type.fields.map(([name,offset,bit,field,presence]) => [name,
      presence ? (view.getUint8(at+presence[0])&(1<<presence[1]) ? readType(field.nullable,at+offset,bit) : null) : readType(field,at+offset,bit)]));
  }
  const value = readStruct(schema, 0);
  if (claimed !== bytes.length) throw new TypeError('trailing Mojom data');
  return value;
}

// The endpoint factory is injectable for tests and alternate typed hosts.
export function createMojomCommandApi(entries, namespace, transport = sendTypedRequest) {
  if (typeof transport !== 'function') throw new TypeError('expected typed Mojo transport');
  const root = Object.create(null);
  for (const [path, ordinal, request, response, interfaceName] of entries) {
    const segments = path.split('.');
    let node = root;
    for (const segment of segments.slice(0, -1)) {
      if (['then', '__proto__', 'constructor', 'prototype'].includes(segment)) throw new TypeError('reserved Mojom path');
      node = node[segment] ||= Object.create(null);
    }
    const leaf = segments.at(-1);
    if (['then', '__proto__', 'constructor', 'prototype'].includes(leaf) || Object.hasOwn(node, leaf)) throw new TypeError('duplicate/reserved Mojom endpoint');
    const endpoint = Object.freeze({interfaceName, ordinal, request, response});
    node[leaf] = async (...args) => {
      if (args.length > request.fields.length || request.fields.slice(args.length).some(([, , , type]) => !type?.nullable)) throw new TypeError('invalid Mojom argument count');
      const value = Object.fromEntries(request.fields.map(([name], index) => [name, args[index]]));
      // Validate the typed data before sending any message. No request envelope.
      encodeMojom(request, value);
      const result = await transport(endpoint, value);
      const decoded = response.fields.length ? result[response.fields[0][0]] : undefined;
      if (interfaceName === 'tuffite.wire.Window') {
        return Array.isArray(decoded) ? decoded.map(windowReference) : windowReference(decoded);
      }
      return decoded;
    };
  }
  function freeze(node) {
    for (const value of Object.values(node)) if (typeof value === 'object') freeze(value);
    return Object.freeze(node);
  }
  return freeze(root);
}

export class TuffiteError extends Error {
  constructor(code, message, options = {}) {
    super(message);
    this.name = 'TuffiteError';
    this.code = code;
    if (options.requestId !== undefined) this.requestId = options.requestId;
    if (options.cause !== undefined) this.cause = options.cause;
  }
}

const transportCodes = Object.freeze({
  1: 'invalid_argument',
  2: 'access_denied',
  3: 'unavailable',
  4: 'cancelled',
  5: 'timed_out',
  6: 'resource_exhausted',
});
const mojoTypes = new WeakMap();
const nativeRemotes = new Map();
const responseTypes = new WeakMap();
function nativeType(schema) {
  if (typeof schema === 'string') {
    if (schema === 'bytes') return mojo.internal.Array(mojo.internal.Uint8, false);
    const names = {bool:'Bool',string:'String',u8:'Uint8',i8:'Int8',u16:'Uint16',i16:'Int16',u32:'Uint32',i32:'Int32',u64:'Uint64',i64:'Int64',f32:'Float',f64:'Double'};
    return mojo.internal[names[schema]];
  }
  if (schema.nullable) return nativeType(schema.nullable);
  if (schema.enum) return mojo.internal.Enum();
  if (schema.array) return mojo.internal.Array(nativeType(schema.array), !!schema.array.nullable);
  if (mojoTypes.has(schema)) return mojoTypes.get(schema);
  const type = {};
  mojoTypes.set(schema,type);
  const fields = [];
  for (const [name,offset,bit,field,presence] of schema.fields) {
    if (presence) {
      const primary = {isPrimary:true,originalFieldName:name,linkedValueFieldName:name+'Value'};
      const secondary = {isPrimary:false,originalFieldName:name,linkedValueFieldName:name+'Present'};
      fields.push(mojo.internal.StructField(name+'Present',presence[0]-8,presence[1],mojo.internal.Bool,false,false,0,primary));
      const fallback = field.nullable?.enum?.[0] ?? (['u64','i64'].includes(field.nullable)?0n:field.nullable==='bool'?false:0);
      fields.push(mojo.internal.StructField(name+'Value',offset-8,bit,nativeType(field.nullable),fallback,false,0,secondary));
    } else {
      fields.push(mojo.internal.StructField(name,offset-8,bit,nativeType(field),null,!!field?.nullable));
    }
  }
  mojo.internal.Struct(type,'Tuffite typed parameters',fields,[[0,schema.size]]);
  return type;
}
function normalizeNative(schema, value) {
  if (value == null) return value;
  if (schema === 'bytes') return new Uint8Array(value);
  if (typeof schema !== 'object') return value;
  if (schema.nullable) return normalizeNative(schema.nullable,value);
  if (schema.array) return value.map(item=>normalizeNative(schema.array,item));
  if (schema.enum) return value;
  return Object.fromEntries(schema.fields.map(([name,,,type])=>[name,normalizeNative(type,value[name])]));
}
async function sendTypedRequest(endpoint, value) {
  if (typeof globalThis.Mojo?.bindInterface !== 'function') {
    throw new TuffiteError('transport_unavailable','Mojo is unavailable for this document');
  }
  let remote = nativeRemotes.get(endpoint.interfaceName);
  if (!remote) {
    const {handle0,handle1} = Mojo.createMessagePipe();
    remote = new mojo.internal.interfaceSupport.InterfaceRemoteBase(null,handle0);
    Mojo.bindInterface(endpoint.interfaceName,handle1);
    remote.getConnectionErrorEventRouter().addListener(()=>nativeRemotes.delete(endpoint.interfaceName));
    nativeRemotes.set(endpoint.interfaceName,remote);
  }
  let responseType=responseTypes.get(endpoint);
  if(!responseType) {
    responseType=nativeType({size:32,fields:[['status',8,0,'i32'],['value',16,0,{nullable:endpoint.response}],['error',24,0,{nullable:'string'}]]});
    responseTypes.set(endpoint,responseType);
  }
  let reply;
  try {
    reply = await remote.sendMessage(endpoint.ordinal,nativeType(endpoint.request),responseType,endpoint.request.fields.map(([name])=>value[name]),false);
  } catch(cause) {
    throw new TuffiteError('transport_unavailable',cause?.message || 'Mojo interface disconnected',{cause});
  }
  if (reply.status !== 0) throw new TuffiteError(transportCodes[reply.status] ?? 'internal',reply.error ?? 'Mojo method failed', {cause:{status:reply.status}});
  if (!reply.value) throw new TuffiteError('decode_error','Missing typed Mojo response');
  const result = normalizeNative(endpoint.response,reply.value);
  encodeMojom(endpoint.response,result);
  return result;
}

function windowReference(info) {
  if (info && typeof info.id === 'bigint') {
    const id = info.id;
    Object.defineProperty(info,'postMessage',{value:(...args)=>globalThis.tuffite.postWindowMessage(id,...args)});
  }
  return info;
}

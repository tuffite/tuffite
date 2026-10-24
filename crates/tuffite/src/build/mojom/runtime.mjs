// Browser SDK for the Tuffite binary Shell Protocol. Applications
// normally consume this through generated bindings instead of calling it.
const encoder = new TextEncoder();
const decoder = new TextDecoder();
const codecDecoder = new TextDecoder('utf-8', {fatal: true});
const MAX_COLLECTION_LENGTH = 16 * 1024 * 1024;
const REQUEST_HEADER_LENGTH = 36;
const RESPONSE_HEADER_LENGTH = 20;
const FRAME_MAGIC = Object.freeze([0x43, 0x45, 0x52, 0x4d]);
const REQUEST_KIND = 1;
const RESPONSE_KIND = 2;
let nextRequestId = 1n;
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

export function createMojomCommandApi(entries, namespace, transport = invokeBinary) {
  if (typeof transport !== 'function') throw new TypeError('expected Mojom transport');
  const root = Object.create(null);
  for (const [path, id, fingerprint, request, response, channel = `${namespace}.${path}`] of entries) {
    const segments = path.split('.');
    let node = root;
    for (const segment of segments.slice(0, -1)) {
      if (['then', '__proto__', 'constructor', 'prototype'].includes(segment)) throw new TypeError('reserved Mojom path');
      node = node[segment] ||= Object.create(null);
    }
    const leaf = segments.at(-1);
    if (['then', '__proto__', 'constructor', 'prototype'].includes(leaf) || Object.hasOwn(node, leaf)) throw new TypeError('duplicate/reserved Mojom endpoint');
    node[leaf] = async (...args) => {
      if (args.length > request.fields.length || request.fields.slice(args.length).some(([, , , type]) => !type?.nullable)) throw new TypeError('invalid Mojom argument count');
      const value = Object.fromEntries(request.fields.map(([name], index) => [name, args[index]]));
      const payload = await transport(channel, BigInt(id), BigInt(fingerprint), encodeMojom(request, value));
      const result = decodeMojom(response, payload);
      const decoded = response.fields.length ? result[response.fields[0][0]] : undefined;
      if (channel.startsWith('tuffite.Window.')) {
        const enrich = info => {
          if (info && typeof info.id === 'bigint') {
            const id = info.id;
            Object.defineProperty(info,'postMessage',{value:(...args)=>globalThis.tuffite.postWindowMessage(id,...args)});
          }
          return info;
        };
        return Array.isArray(decoded) ? decoded.map(enrich) : enrich(decoded);
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
  2: 'incompatible_abi',
  3: 'invalid_handle',
  4: 'unavailable',
  5: 'resource_exhausted',
  6: 'internal',
});
const commandCodes = Object.freeze({
  1: 'invalid_request',
  2: 'unknown_command',
  3: 'handler_failed',
  4: 'invalid_argument',
  5: 'schema_mismatch',
});

async function invokeNative(channel, payload) {
  const bridge = globalThis.tuffite;
  if (!bridge || typeof bridge.invoke !== 'function') {
    throw new Error('Tuffite native bridge is unavailable for this document');
  }
  let response;
  try {
    response = await bridge.invoke(channel, payload);
  } catch (cause) {
    if (cause instanceof TuffiteError) throw cause;
    const status = Number.isInteger(cause?.status) ? cause.status : undefined;
    throw new TuffiteError(
      cause?.code ?? transportCodes[status] ?? 'transport_unavailable',
      cause?.message ?? 'ShellAPI transport unavailable',
      {cause},
    );
  }
  if (!(response instanceof Uint8Array)) {
    throw new TypeError('Tuffite native bridge returned a non-binary response');
  }
  return response;
}

// Framework routes already carry request identity and RPC status in dispatch.
export async function invokeFrameworkMojom(channel, _id, _fingerprint, payload) {
  return invokeNative(channel, payload);
}

export async function invokeBinary(channel, commandId, schemaFingerprint, payload = new Uint8Array()) {
  if (typeof channel !== 'string' || !channel.includes('.')) {
    throw new TypeError('ShellAPI channel must be a fully qualified API path');
  }
  if (!(payload instanceof Uint8Array)) {
    throw new TypeError('ShellAPI payload must be a Uint8Array');
  }
  const requestId = nextRequestId++;
  const body = new Uint8Array(REQUEST_HEADER_LENGTH + payload.byteLength);
  body.set(FRAME_MAGIC);
  const view = new DataView(body.buffer);
  view.setUint16(4, REQUEST_KIND, true);
  view.setBigUint64(8, requestId, true);
  view.setBigUint64(16, commandId, true);
  view.setBigUint64(24, schemaFingerprint, true);
  view.setUint32(32, payload.byteLength, true);
  body.set(payload, REQUEST_HEADER_LENGTH);
  const response = await invokeNative(channel, body);
  if (response.byteLength < RESPONSE_HEADER_LENGTH || decoder.decode(response.subarray(0, 4)) !== 'CERM') {
    throw new TuffiteError('decode_error', 'Malformed ShellAPI response', {requestId});
  }
  const responseView = new DataView(response.buffer, response.byteOffset);
  if (responseView.getUint16(4, true) !== RESPONSE_KIND ||
      responseView.getBigUint64(8, true) !== requestId ||
      responseView.getUint32(16, true) !== response.byteLength - RESPONSE_HEADER_LENGTH) {
    throw new TuffiteError('protocol_mismatch', 'Mismatched ShellAPI response', {requestId});
  }
  const status = responseView.getUint16(6, true);
  if (status !== 0) {
    throw new TuffiteError(
      commandCodes[status] ?? 'command_failed',
      decoder.decode(response.subarray(RESPONSE_HEADER_LENGTH)) || `command failed (${status})`,
      {requestId},
    );
  }
  return response.subarray(RESPONSE_HEADER_LENGTH);
}

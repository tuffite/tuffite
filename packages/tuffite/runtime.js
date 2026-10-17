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

export function createCommandApi(entries, typed = false, rootTarget, channelPrefix = '') {
  const root = Object.create(null);
  for (const [path, id] of entries) {
    const segments = path.split('.');
    let node = root;
    for (const segment of segments.slice(0, -1)) {
      node = node[segment] ||= Object.create(null);
    }
    node[segments.at(-1)] = typeof id === 'bigint' ? Object.freeze({
      raw: true,
      id,
      channel: channelPrefix ? `${channelPrefix}.${path}` : path,
    }) : id;
  }
  const facades = new WeakMap();
  const facade = (node) => {
    const existing = facades.get(node);
    if (existing) return existing;
    const values = new Map();
    const proxy = new Proxy(Object.create(null), {
    get(_target, property) {
      if (typeof property !== 'string' || !Object.hasOwn(node, property)) return undefined;
      if (values.has(property)) return values.get(property);
      const value = node[property];
      let exposed;
      if (value && value.raw === true) {
        exposed = (payload = new Uint8Array()) =>
          invokeBinary(value.channel, value.id, value.schemaFingerprint ?? 0n, payload);
      } else if (typed && value && typeof value.id === 'bigint') {
        exposed = async (...args) => {
          validateArgumentCount(value.request, args);
          const writer = new BinaryWriter();
          value.request.forEach((schema, index) => encodeValue(writer, schema, args[index]));
          const payload = await invokeBinary(
            value.channel,
            value.id,
            value.schemaFingerprint,
            writer.finish(),
          );
          const reader = new BinaryReader(payload);
          const response = decodeValue(reader, value.response);
          reader.finish();
          return response;
        };
      } else if (typed && value && typeof value.invoke === 'function') {
        exposed = async (...args) => {
          validateArgumentCount(value.request, args);
          validateValues(value.request, args);
          const response = await value.invoke(...args);
          validateValue(value.response, response);
          return response;
        };
      } else if (typed && value && value.constant === true) {
        exposed = value.value;
      } else {
        exposed = facade(value);
      }
      values.set(property, exposed);
      return exposed;
    },
    set() { return false; },
    defineProperty() { return false; },
    deleteProperty() { return false; },
    });
    facades.set(node, proxy);
    return proxy;
  };
  const api = facade(root);
  if (!rootTarget) return api;
  for (const property of Object.keys(root)) {
    if (Object.hasOwn(rootTarget, property)) {
      throw new TypeError(`ShellAPI path conflicts with native bridge: ${property}`);
    }
    Object.defineProperty(rootTarget, property, {
      configurable: false,
      enumerable: true,
      get: () => api[property],
    });
  }
  return rootTarget;
}

function validateArgumentCount(schemas, args) {
  const required = schemas.findIndex((schema) => schema.kind === 'optional');
  const minimum = required === -1 ? schemas.length : required;
  if (args.length < minimum || args.length > schemas.length) {
    const expected = minimum === schemas.length ? `${schemas.length}` :
      `${minimum}-${schemas.length}`;
    throw new TypeError(`expected ${expected} ShellAPI arguments`);
  }
}

function validateValues(schemas, values) {
  const writer = new BinaryWriter();
  schemas.forEach((schema, index) => encodeValue(writer, schema, values[index]));
}

function validateValue(schema, value) {
  const writer = new BinaryWriter();
  encodeValue(writer, schema, value);
}

class BinaryWriter {
  #chunks = []; #length = 0;
  append(bytes) {
    if (!(bytes instanceof Uint8Array) ||
        this.#length + bytes.byteLength > MAX_COLLECTION_LENGTH) {
      throw new TypeError('ShellAPI request payload is too large');
    }
    this.#chunks.push(bytes);
    this.#length += bytes.byteLength;
  }
  u8(value) { this.append(Uint8Array.of(value)); }
  u32(value) {
    const bytes = new Uint8Array(4);
    new DataView(bytes.buffer).setUint32(0, value, true);
    this.append(bytes);
  }
  i64(value) {
    const integer = typeof value === 'bigint' ? value : BigInt(value);
    if (BigInt.asIntN(64, integer) !== integer) throw new TypeError('bigint is outside i64 range');
    const bytes = new Uint8Array(8);
    new DataView(bytes.buffer).setBigInt64(0, integer, true);
    this.append(bytes);
  }
  f64(value) {
    const bytes = new Uint8Array(8);
    new DataView(bytes.buffer).setFloat64(0, value, true);
    this.append(bytes);
  }
  bytes(value) {
    if (!(value instanceof Uint8Array)) throw new TypeError('expected Uint8Array');
    if (value.byteLength > MAX_COLLECTION_LENGTH) throw new TypeError('binary value is too large');
    this.u32(value.byteLength);
    this.append(value);
  }
  finish() {
    const result = new Uint8Array(this.#length);
    let offset = 0;
    for (const chunk of this.#chunks) { result.set(chunk, offset); offset += chunk.byteLength; }
    return result;
  }
}

class BinaryReader {
  #bytes; #offset = 0;
  constructor(bytes) {
    if (!(bytes instanceof Uint8Array) || bytes.byteLength > MAX_COLLECTION_LENGTH) {
      throw new TypeError('invalid ShellAPI response payload');
    }
    this.#bytes = bytes;
  }
  take(length) {
    if (!Number.isSafeInteger(length) || length < 0 || this.#offset + length > this.#bytes.byteLength) {
      throw new TypeError('invalid ShellAPI response payload');
    }
    const value = this.#bytes.subarray(this.#offset, this.#offset + length);
    this.#offset += length;
    return value;
  }
  u8() { return this.take(1)[0]; }
  u32() { const value = this.take(4); return new DataView(value.buffer, value.byteOffset, 4).getUint32(0, true); }
  i64() { const value = this.take(8); return new DataView(value.buffer, value.byteOffset, 8).getBigInt64(0, true); }
  f64() { const value = this.take(8); return new DataView(value.buffer, value.byteOffset, 8).getFloat64(0, true); }
  bytes() {
    const length = this.u32();
    if (length > MAX_COLLECTION_LENGTH) throw new TypeError('binary value is too large');
    return this.take(length).slice();
  }
  finish() { if (this.#offset !== this.#bytes.byteLength) throw new TypeError('trailing ShellAPI response data'); }
}

function encodeValue(writer, schema, value) {
  switch (schema.kind) {
    case 'void': return;
    case 'boolean':
      if (typeof value !== 'boolean') throw new TypeError('expected boolean');
      writer.u8(value ? 1 : 0); return;
    case 'number':
      if (typeof value !== 'number') throw new TypeError('expected number');
      writer.f64(value); return;
    case 'integer':
      if (!Number.isSafeInteger(value)) throw new TypeError('expected safe integer');
      writer.i64(BigInt(value)); return;
    case 'bigint':
      if (typeof value !== 'bigint') throw new TypeError('expected bigint');
      writer.i64(value); return;
    case 'string':
      if (typeof value !== 'string') throw new TypeError('expected string');
      writer.bytes(encoder.encode(value)); return;
    case 'bytes': writer.bytes(value); return;
    case 'date': {
      if (!(value instanceof Date) || !Number.isSafeInteger(value.getTime())) {
        throw new TypeError('expected valid Date');
      }
      writer.i64(BigInt(value.getTime())); return;
    }
    case 'optional':
      writer.u8(value === null || value === undefined ? 0 : 1);
      if (value !== null && value !== undefined) encodeValue(writer, schema.value, value);
      return;
    case 'array':
      if (!Array.isArray(value) || value.length > MAX_COLLECTION_LENGTH) {
        throw new TypeError('expected bounded array');
      }
      writer.u32(value.length);
      for (const item of value) encodeValue(writer, schema.item, item);
      return;
    case 'object':
      if (value === null || typeof value !== 'object' || Array.isArray(value)) {
        throw new TypeError('expected object');
      }
      for (const [name, field] of schema.fields) encodeValue(writer, field, value[name]);
      return;
    default: throw new TypeError(`unsupported ShellAPI codec ${schema.kind}`);
  }
}

function decodeValue(reader, schema) {
  switch (schema.kind) {
    case 'void': return undefined;
    case 'boolean': { const value = reader.u8(); if (value > 1) throw new TypeError('invalid boolean'); return value === 1; }
    case 'number': return reader.f64();
    case 'integer': {
      const value = Number(reader.i64());
      if (!Number.isSafeInteger(value)) throw new TypeError('integer is outside JavaScript safe range');
      return value;
    }
    case 'bigint': return reader.i64();
    case 'string': return codecDecoder.decode(reader.bytes());
    case 'bytes': return reader.bytes();
    case 'date': {
      const milliseconds = reader.i64();
      const value = Number(milliseconds);
      if (!Number.isSafeInteger(value)) throw new TypeError('Date is outside JavaScript range');
      const date = new Date(value);
      if (Number.isNaN(date.getTime())) throw new TypeError('invalid Date');
      return date;
    }
    case 'optional': {
      const tag = reader.u8();
      if (tag === 0) return undefined;
      if (tag !== 1) throw new TypeError('invalid optional tag');
      return decodeValue(reader, schema.value);
    }
    case 'array': {
      const length = reader.u32();
      if (length > MAX_COLLECTION_LENGTH) throw new TypeError('array is too large');
      return Array.from({length}, () => decodeValue(reader, schema.item));
    }
    case 'object': return Object.fromEntries(schema.fields.map(([name, field]) => [name, decodeValue(reader, field)]));
    default: throw new TypeError(`unsupported ShellAPI codec ${schema.kind}`);
  }
}

export function createTypedCommandApi(entries, constants = [], namespace = '') {
  return createCommandApi([
    ...entries.map(([path, id, schemaFingerprint, request, response]) => [
      path,
      Object.freeze({
        id: BigInt(id),
        schemaFingerprint: BigInt(schemaFingerprint),
        channel: namespace ? `${namespace}.${path}` : path,
        request,
        response,
      }),
    ]),
    ...constants.map(([path, value]) => [
      path,
      Object.freeze({constant: true, value}),
    ]),
  ], true);
}

export function createFrameworkApi(entries, constants = []) {
  const native = globalThis.tuffite;
  if (!native || typeof native.invoke !== 'function') {
    throw new Error('Tuffite native bridge is unavailable for this document');
  }
  return createCommandApi([
    ...entries.map(([path, request, response]) => {
      const invoke = async (...args) => {
        validateArgumentCount(request, args);
        const writer = new BinaryWriter();
        request.forEach((schema, index) => encodeValue(writer, schema, args[index]));
        const payload = await invokeNative(`tuffite.${path}`, writer.finish());
        const reader = new BinaryReader(payload);
        const value = decodeValue(reader, response);
        reader.finish();
        if (path.startsWith("window.")) {
          const enrich = info => {
            if (info && typeof info.id === "bigint") {
              const id = info.id;
              Object.defineProperty(info, "postMessage", {
                value: (...args) => native.postWindowMessage(id, ...args),
              });
            }
            return info;
          };
          return Array.isArray(value) ? value.map(enrich) : enrich(value);
        }
        return value;
      };
      return [path, Object.freeze({invoke, request, response})];
    }),
    ...constants.map(([path, value]) => [
      path,
      Object.freeze({constant: true, value}),
    ]),
  ], true, native);
}

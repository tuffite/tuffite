// Generated from Mojom v0. Do not edit.
const encoder = new TextEncoder();
const decoder = new TextDecoder();
const codecDecoder = new TextDecoder('utf-8', { fatal: true });
const MAX_COLLECTION_LENGTH = 16 * 1024 * 1024;
const REQUEST_HEADER_LENGTH = 36;
const RESPONSE_HEADER_LENGTH = 20;
const FRAME_MAGIC = Object.freeze([
  67,
  69,
  82,
  77
]);
const REQUEST_KIND = 1;
const RESPONSE_KIND = 2;
let nextRequestId = 1n;
const mojomScalars = Object.freeze(Object.fromEntries([
  'u8',
  'i8',
  'u16',
  'i16',
  'u32',
  'i32',
  'u64',
  'i64',
  'f32',
  'f64'
].map((type) => {
  const bits = Number(type.slice(1));
  const unsigned = type[0] === 'u';
  const suffix = `${bits === 64 && type[0] !== 'f' ? 'Big' : ''}${type[0] === 'f' ? 'Float' : unsigned ? 'Uint' : 'Int'}${bits}`;
  return [type, {
    get: `get${suffix}`,
    set: `set${suffix}`,
    min: unsigned ? 0 : -(2 ** (bits - 1)),
    max: unsigned ? 2 ** bits - 1 : 2 ** (bits - 1) - 1
  }];
})));
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
      next.set(buffer);
      buffer = next;
      dataView = new DataView(buffer.buffer);
    }
    return at;
  }
  function writeType(field, item, at, bit = 0) {
    if (field && typeof field === 'object') {
      if ('nullable' in field) {
        if (item == null) return;
        return writeType(field.nullable, item, at, bit);
      }
      if ('enum' in field) {
        if (!field.enum.includes(item)) throw new TypeError('invalid Mojom enum');
        view().setInt32(at, item, true);
        return;
      }
      if ('array' in field) {
        if (!Array.isArray(item) || item.length > MAX_COLLECTION_LENGTH) throw new TypeError('expected bounded Mojom array');
        const size = 8 + (field.boolean ? Math.ceil(item.length / 8) : item.length * field.width);
        const target = allocate(size);
        view().setBigUint64(at, BigInt(target - at), true);
        view().setUint32(target, size, true);
        view().setUint32(target + 4, item.length, true);
        item.forEach((value, index) => writeType(field.array, value, target + 8 + (field.boolean ? Math.floor(index / 8) : index * field.width), field.boolean ? index % 8 : 0));
        return;
      }
      const target = allocate(field.size);
      view().setBigUint64(at, BigInt(target - at), true);
      writeStruct(field, item, target);
      return;
    }
    if (field === 'string' || field === 'bytes') {
      if (field === 'string' ? typeof item !== 'string' : !(item instanceof Uint8Array)) throw new TypeError(`invalid Mojom ${field}`);
      const bytes = field === 'string' ? encoder.encode(item) : item;
      const target = allocate(8 + bytes.length);
      view().setBigUint64(at, BigInt(target - at), true);
      view().setUint32(target, 8 + bytes.length, true);
      view().setUint32(target + 4, bytes.length, true);
      buffer.set(bytes, target + 8);
      return;
    }
    if (field === 'bool') {
      if (typeof item !== 'boolean') throw new TypeError('expected boolean');
      buffer[at] |= Number(item) << bit;
      return;
    }
    const scalar = mojomScalars[field];
    if (field === 'u64' || field === 'i64') {
      if (typeof item !== 'bigint' || (field === 'u64' ? BigInt.asUintN(64, item) : BigInt.asIntN(64, item)) !== item) throw new TypeError('expected bounded bigint');
      view()[scalar.set](at, item, true);
    } else if (field[0] === 'f') {
      if (typeof item !== 'number') throw new TypeError('expected number');
      view()[scalar.set](at, item, true);
    } else {
      if (!scalar || !Number.isInteger(item) || item < scalar.min || item > scalar.max) throw new TypeError(`expected ${field}`);
      view()[scalar.set](at, item, true);
    }
  }
  function writeStruct(type, object, base) {
    if (!object || typeof object !== 'object' || Array.isArray(object)) throw new TypeError('expected Mojom struct');
    view().setUint32(base, type.size, true);
    for (const [name, offset, bit, field, presence] of type.fields) {
      const item = object[name];
      if (presence) {
        if (item == null) continue;
        buffer[base + presence[0]] |= 1 << presence[1];
        writeType(field.nullable, item, base + offset, bit);
      } else writeType(field, item, base + offset, bit);
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
      if ('nullable' in field) return view.getBigUint64(at, true) === 0n ? null : readType(field.nullable, at, bit);
      if ('enum' in field) {
        const value = view.getInt32(at, true);
        if (!field.enum.includes(value)) throw new TypeError('unknown Mojom enum');
        return value;
      }
      if ('array' in field) {
        const target = pointer(at), size = view.getUint32(target, true), length = view.getUint32(target + 4, true);
        if (length > MAX_COLLECTION_LENGTH || size !== 8 + (field.boolean ? Math.ceil(length / 8) : length * field.width)) throw new TypeError('invalid Mojom array');
        claim(target, size);
        return Array.from({ length }, (_, index) => readType(field.array, target + 8 + (field.boolean ? Math.floor(index / 8) : index * field.width), field.boolean ? index % 8 : 0));
      }
      return readStruct(field, pointer(at));
    }
    if (field === 'string' || field === 'bytes') {
      const target = pointer(at), size = view.getUint32(target, true), length = view.getUint32(target + 4, true);
      if (size !== length + 8) throw new TypeError('invalid Mojom array header');
      claim(target, size);
      const data = bytes.subarray(target + 8, target + size);
      return field === 'string' ? codecDecoder.decode(data) : data.slice();
    }
    if (field === 'bool') return (view.getUint8(at) & 1 << bit) !== 0;
    return view[mojomScalars[field].get](at, true);
  }
  function readStruct(type, at) {
    claim(at, type.size);
    if (view.getUint32(at, true) !== type.size || view.getUint32(at + 4, true) !== 0) throw new TypeError('unsupported Mojom struct version/size');
    return Object.fromEntries(type.fields.map(([name, offset, bit, field, presence]) => [name, presence ? view.getUint8(at + presence[0]) & 1 << presence[1] ? readType(field.nullable, at + offset, bit) : null : readType(field, at + offset, bit)]));
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
      if ([
        'then',
        '__proto__',
        'constructor',
        'prototype'
      ].includes(segment)) throw new TypeError('reserved Mojom path');
      node = node[segment] ||= Object.create(null);
    }
    const leaf = segments.at(-1);
    if ([
      'then',
      '__proto__',
      'constructor',
      'prototype'
    ].includes(leaf) || Object.hasOwn(node, leaf)) throw new TypeError('duplicate/reserved Mojom endpoint');
    node[leaf] = async (...args) => {
      if (args.length > request.fields.length || request.fields.slice(args.length).some(([, , , type]) => !type?.nullable)) throw new TypeError('invalid Mojom argument count');
      const value = Object.fromEntries(request.fields.map(([name], index) => [name, args[index]]));
      const payload = await transport(channel, BigInt(id), BigInt(fingerprint), encodeMojom(request, value));
      const result = decodeMojom(response, payload);
      const decoded = response.fields.length ? result[response.fields[0][0]] : undefined;
      if (channel.startsWith('tuffite.Window.')) {
        const enrich = (info) => {
          if (info && typeof info.id === 'bigint') {
            const id = info.id;
            Object.defineProperty(info, 'postMessage', { value: (...args) => globalThis.tuffite.postWindowMessage(id, ...args) });
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
  6: 'internal'
});
const commandCodes = Object.freeze({
  1: 'invalid_request',
  2: 'unknown_command',
  3: 'handler_failed',
  4: 'invalid_argument',
  5: 'schema_mismatch'
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
    throw new TuffiteError(cause?.code ?? transportCodes[status] ?? 'transport_unavailable', cause?.message ?? 'ShellAPI transport unavailable', { cause });
  }
  if (!(response instanceof Uint8Array)) {
    throw new TypeError('Tuffite native bridge returned a non-binary response');
  }
  return response;
}
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
    throw new TuffiteError('decode_error', 'Malformed ShellAPI response', { requestId });
  }
  const responseView = new DataView(response.buffer, response.byteOffset);
  if (responseView.getUint16(4, true) !== RESPONSE_KIND || responseView.getBigUint64(8, true) !== requestId || responseView.getUint32(16, true) !== response.byteLength - RESPONSE_HEADER_LENGTH) {
    throw new TuffiteError('protocol_mismatch', 'Mismatched ShellAPI response', { requestId });
  }
  const status = responseView.getUint16(6, true);
  if (status !== 0) {
    throw new TuffiteError(commandCodes[status] ?? 'command_failed', decoder.decode(response.subarray(RESPONSE_HEADER_LENGTH)) || `command failed (${status})`, { requestId });
  }
  return response.subarray(RESPONSE_HEADER_LENGTH);
}
export const mojomTypes = {
  'DialogOpenDirectoryRequest': {
    'fields': [[
      'options',
      8,
      0,
      { 'nullable': {
        'fields': [[
          'title',
          8,
          0,
          { 'nullable': 'string' }
        ]],
        'size': 16
      } }
    ]],
    'size': 16
  },
  'DialogOpenDirectoryResponse': {
    'fields': [[
      'result',
      8,
      0,
      { 'nullable': 'string' }
    ]],
    'size': 16
  },
  'DialogOpenFileRequest': {
    'fields': [[
      'options',
      8,
      0,
      { 'nullable': {
        'fields': [
          [
            'title',
            8,
            0,
            { 'nullable': 'string' }
          ],
          [
            'defaultPath',
            16,
            0,
            { 'nullable': 'string' }
          ],
          [
            'extensions',
            24,
            0,
            { 'nullable': {
              'array': {
                'array': 'string',
                'boolean': false,
                'width': 8
              },
              'boolean': false,
              'width': 8
            } }
          ]
        ],
        'size': 32
      } }
    ]],
    'size': 16
  },
  'DialogOpenFileResponse': {
    'fields': [[
      'result',
      8,
      0,
      { 'nullable': 'string' }
    ]],
    'size': 16
  },
  'FsCreateDirRequest': {
    'fields': [[
      'base',
      8,
      0,
      { 'enum': [
        0,
        1,
        2,
        3
      ] }
    ], [
      'path',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'FsCreateDirResponse': {
    'fields': [],
    'size': 8
  },
  'FsExistsRequest': {
    'fields': [[
      'base',
      8,
      0,
      { 'enum': [
        0,
        1,
        2,
        3
      ] }
    ], [
      'path',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'FsExistsResponse': {
    'fields': [[
      'result',
      8,
      0,
      'bool'
    ]],
    'size': 16
  },
  'FsReadFileRequest': {
    'fields': [[
      'base',
      8,
      0,
      { 'enum': [
        0,
        1,
        2,
        3
      ] }
    ], [
      'path',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'FsReadFileResponse': {
    'fields': [[
      'result',
      8,
      0,
      'bytes'
    ]],
    'size': 16
  },
  'FsRemoveFileRequest': {
    'fields': [[
      'base',
      8,
      0,
      { 'enum': [
        0,
        1,
        2,
        3
      ] }
    ], [
      'path',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'FsRemoveFileResponse': {
    'fields': [],
    'size': 8
  },
  'FsWriteFileRequest': {
    'fields': [
      [
        'base',
        8,
        0,
        { 'enum': [
          0,
          1,
          2,
          3
        ] }
      ],
      [
        'path',
        16,
        0,
        'string'
      ],
      [
        'contents',
        24,
        0,
        'bytes'
      ]
    ],
    'size': 32
  },
  'FsWriteFileResponse': {
    'fields': [],
    'size': 8
  },
  'LogDebugRequest': {
    'fields': [[
      'message',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'LogDebugResponse': {
    'fields': [],
    'size': 8
  },
  'LogErrorRequest': {
    'fields': [[
      'message',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'LogErrorResponse': {
    'fields': [],
    'size': 8
  },
  'LogInfoRequest': {
    'fields': [[
      'message',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'LogInfoResponse': {
    'fields': [],
    'size': 8
  },
  'LogTraceRequest': {
    'fields': [[
      'message',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'LogTraceResponse': {
    'fields': [],
    'size': 8
  },
  'LogWarnRequest': {
    'fields': [[
      'message',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'LogWarnResponse': {
    'fields': [],
    'size': 8
  },
  'NetRequestRequest': {
    'fields': [[
      'url',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'NetRequestResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [[
          'status',
          8,
          0,
          'f64'
        ], [
          'body',
          16,
          0,
          'bytes'
        ]],
        'size': 24
      }
    ]],
    'size': 16
  },
  'NetworkResponse': {
    'fields': [[
      'status',
      8,
      0,
      'f64'
    ], [
      'body',
      16,
      0,
      'bytes'
    ]],
    'size': 24
  },
  'OpenDirectoryDialogOptions': {
    'fields': [[
      'title',
      8,
      0,
      { 'nullable': 'string' }
    ]],
    'size': 16
  },
  'OpenFileDialogOptions': {
    'fields': [
      [
        'title',
        8,
        0,
        { 'nullable': 'string' }
      ],
      [
        'defaultPath',
        16,
        0,
        { 'nullable': 'string' }
      ],
      [
        'extensions',
        24,
        0,
        { 'nullable': {
          'array': {
            'array': 'string',
            'boolean': false,
            'width': 8
          },
          'boolean': false,
          'width': 8
        } }
      ]
    ],
    'size': 32
  },
  'PathDirectories': {
    'fields': [
      [
        'resource',
        8,
        0,
        'string'
      ],
      [
        'data',
        16,
        0,
        'string'
      ],
      [
        'cache',
        24,
        0,
        'string'
      ],
      [
        'temp',
        32,
        0,
        'string'
      ]
    ],
    'size': 40
  },
  'PathDirectoriesRequest': {
    'fields': [],
    'size': 8
  },
  'PathDirectoriesResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'resource',
            8,
            0,
            'string'
          ],
          [
            'data',
            16,
            0,
            'string'
          ],
          [
            'cache',
            24,
            0,
            'string'
          ],
          [
            'temp',
            32,
            0,
            'string'
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'PathResolveRequest': {
    'fields': [[
      'base',
      8,
      0,
      { 'enum': [
        0,
        1,
        2,
        3
      ] }
    ], [
      'relative',
      16,
      0,
      { 'nullable': 'string' }
    ]],
    'size': 24
  },
  'PathResolveResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'RemotingCapabilities': {
    'fields': [
      [
        'available',
        8,
        0,
        'bool'
      ],
      [
        'protocol',
        16,
        0,
        'string'
      ],
      [
        'videoCodecs',
        24,
        0,
        {
          'array': 'string',
          'boolean': false,
          'width': 8
        }
      ],
      [
        'input',
        8,
        1,
        'bool'
      ],
      [
        'clipboard',
        8,
        2,
        'bool'
      ],
      [
        'reconnect',
        8,
        3,
        'bool'
      ],
      [
        'snapshot',
        8,
        4,
        'bool'
      ]
    ],
    'size': 32
  },
  'RemotingCapabilitiesRequest': {
    'fields': [],
    'size': 8
  },
  'RemotingCapabilitiesResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'available',
            8,
            0,
            'bool'
          ],
          [
            'protocol',
            16,
            0,
            'string'
          ],
          [
            'videoCodecs',
            24,
            0,
            {
              'array': 'string',
              'boolean': false,
              'width': 8
            }
          ],
          [
            'input',
            8,
            1,
            'bool'
          ],
          [
            'clipboard',
            8,
            2,
            'bool'
          ],
          [
            'reconnect',
            8,
            3,
            'bool'
          ],
          [
            'snapshot',
            8,
            4,
            'bool'
          ]
        ],
        'size': 32
      }
    ]],
    'size': 16
  },
  'RemotingCaptureRequest': {
    'fields': [[
      'screenId',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 0]
    ]],
    'size': 24
  },
  'RemotingCaptureResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'width',
            8,
            0,
            'f64'
          ],
          [
            'height',
            16,
            0,
            'f64'
          ],
          [
            'mimeType',
            24,
            0,
            'string'
          ],
          [
            'data',
            32,
            0,
            'bytes'
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'RemotingIceCandidate': {
    'fields': [
      [
        'mid',
        8,
        0,
        'string'
      ],
      [
        'mLineIndex',
        16,
        0,
        'f64'
      ],
      [
        'candidate',
        24,
        0,
        'string'
      ]
    ],
    'size': 32
  },
  'RemotingIceServer': {
    'fields': [
      [
        'urls',
        8,
        0,
        {
          'array': 'string',
          'boolean': false,
          'width': 8
        }
      ],
      [
        'username',
        16,
        0,
        { 'nullable': 'string' }
      ],
      [
        'credential',
        24,
        0,
        { 'nullable': 'string' }
      ]
    ],
    'size': 32
  },
  'RemotingInjectInputRequest': {
    'fields': [[
      'event',
      8,
      0,
      {
        'fields': [
          [
            'kind',
            8,
            0,
            'string'
          ],
          [
            'pressed',
            16,
            1,
            { 'nullable': 'bool' },
            [16, 0]
          ],
          [
            'usbKeycode',
            24,
            0,
            { 'nullable': 'f64' },
            [16, 2]
          ],
          [
            'text',
            32,
            0,
            { 'nullable': 'string' }
          ],
          [
            'x',
            40,
            0,
            { 'nullable': 'f64' },
            [16, 3]
          ],
          [
            'y',
            48,
            0,
            { 'nullable': 'f64' },
            [16, 4]
          ],
          [
            'button',
            56,
            0,
            { 'nullable': 'string' }
          ],
          [
            'buttonDown',
            16,
            6,
            { 'nullable': 'bool' },
            [16, 5]
          ],
          [
            'wheelDeltaX',
            64,
            0,
            { 'nullable': 'f64' },
            [16, 7]
          ],
          [
            'wheelDeltaY',
            72,
            0,
            { 'nullable': 'f64' },
            [17, 0]
          ],
          [
            'wheelTicksX',
            80,
            0,
            { 'nullable': 'f64' },
            [17, 1]
          ],
          [
            'wheelTicksY',
            88,
            0,
            { 'nullable': 'f64' },
            [17, 2]
          ],
          [
            'clipboardMimeType',
            96,
            0,
            { 'nullable': 'string' }
          ],
          [
            'clipboardData',
            104,
            0,
            { 'nullable': 'bytes' }
          ]
        ],
        'size': 112
      }
    ]],
    'size': 16
  },
  'RemotingInjectInputResponse': {
    'fields': [],
    'size': 8
  },
  'RemotingInputEvent': {
    'fields': [
      [
        'kind',
        8,
        0,
        'string'
      ],
      [
        'pressed',
        16,
        1,
        { 'nullable': 'bool' },
        [16, 0]
      ],
      [
        'usbKeycode',
        24,
        0,
        { 'nullable': 'f64' },
        [16, 2]
      ],
      [
        'text',
        32,
        0,
        { 'nullable': 'string' }
      ],
      [
        'x',
        40,
        0,
        { 'nullable': 'f64' },
        [16, 3]
      ],
      [
        'y',
        48,
        0,
        { 'nullable': 'f64' },
        [16, 4]
      ],
      [
        'button',
        56,
        0,
        { 'nullable': 'string' }
      ],
      [
        'buttonDown',
        16,
        6,
        { 'nullable': 'bool' },
        [16, 5]
      ],
      [
        'wheelDeltaX',
        64,
        0,
        { 'nullable': 'f64' },
        [16, 7]
      ],
      [
        'wheelDeltaY',
        72,
        0,
        { 'nullable': 'f64' },
        [17, 0]
      ],
      [
        'wheelTicksX',
        80,
        0,
        { 'nullable': 'f64' },
        [17, 1]
      ],
      [
        'wheelTicksY',
        88,
        0,
        { 'nullable': 'f64' },
        [17, 2]
      ],
      [
        'clipboardMimeType',
        96,
        0,
        { 'nullable': 'string' }
      ],
      [
        'clipboardData',
        104,
        0,
        { 'nullable': 'bytes' }
      ]
    ],
    'size': 112
  },
  'RemotingProcessSignalRequest': {
    'fields': [[
      'id',
      8,
      0,
      'i64'
    ], [
      'signal',
      16,
      0,
      {
        'fields': [
          [
            'sequence',
            8,
            0,
            'i64'
          ],
          [
            'description',
            16,
            0,
            { 'nullable': {
              'fields': [[
                'type',
                8,
                0,
                'string'
              ], [
                'sdp',
                16,
                0,
                'string'
              ]],
              'size': 24
            } }
          ],
          [
            'candidates',
            24,
            0,
            {
              'array': {
                'fields': [
                  [
                    'mid',
                    8,
                    0,
                    'string'
                  ],
                  [
                    'mLineIndex',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'candidate',
                    24,
                    0,
                    'string'
                  ]
                ],
                'size': 32
              },
              'boolean': false,
              'width': 8
            }
          ]
        ],
        'size': 32
      }
    ]],
    'size': 24
  },
  'RemotingProcessSignalResponse': {
    'fields': [],
    'size': 8
  },
  'RemotingSendInputRequest': {
    'fields': [[
      'id',
      8,
      0,
      'i64'
    ], [
      'event',
      16,
      0,
      {
        'fields': [
          [
            'kind',
            8,
            0,
            'string'
          ],
          [
            'pressed',
            16,
            1,
            { 'nullable': 'bool' },
            [16, 0]
          ],
          [
            'usbKeycode',
            24,
            0,
            { 'nullable': 'f64' },
            [16, 2]
          ],
          [
            'text',
            32,
            0,
            { 'nullable': 'string' }
          ],
          [
            'x',
            40,
            0,
            { 'nullable': 'f64' },
            [16, 3]
          ],
          [
            'y',
            48,
            0,
            { 'nullable': 'f64' },
            [16, 4]
          ],
          [
            'button',
            56,
            0,
            { 'nullable': 'string' }
          ],
          [
            'buttonDown',
            16,
            6,
            { 'nullable': 'bool' },
            [16, 5]
          ],
          [
            'wheelDeltaX',
            64,
            0,
            { 'nullable': 'f64' },
            [16, 7]
          ],
          [
            'wheelDeltaY',
            72,
            0,
            { 'nullable': 'f64' },
            [17, 0]
          ],
          [
            'wheelTicksX',
            80,
            0,
            { 'nullable': 'f64' },
            [17, 1]
          ],
          [
            'wheelTicksY',
            88,
            0,
            { 'nullable': 'f64' },
            [17, 2]
          ],
          [
            'clipboardMimeType',
            96,
            0,
            { 'nullable': 'string' }
          ],
          [
            'clipboardData',
            104,
            0,
            { 'nullable': 'bytes' }
          ]
        ],
        'size': 112
      }
    ]],
    'size': 24
  },
  'RemotingSendInputResponse': {
    'fields': [],
    'size': 8
  },
  'RemotingSession': {
    'fields': [[
      'id',
      8,
      0,
      'i64'
    ], [
      'state',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'RemotingSessionDescription': {
    'fields': [[
      'type',
      8,
      0,
      'string'
    ], [
      'sdp',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'RemotingSignal': {
    'fields': [
      [
        'sequence',
        8,
        0,
        'i64'
      ],
      [
        'description',
        16,
        0,
        { 'nullable': {
          'fields': [[
            'type',
            8,
            0,
            'string'
          ], [
            'sdp',
            16,
            0,
            'string'
          ]],
          'size': 24
        } }
      ],
      [
        'candidates',
        24,
        0,
        {
          'array': {
            'fields': [
              [
                'mid',
                8,
                0,
                'string'
              ],
              [
                'mLineIndex',
                16,
                0,
                'f64'
              ],
              [
                'candidate',
                24,
                0,
                'string'
              ]
            ],
            'size': 32
          },
          'boolean': false,
          'width': 8
        }
      ]
    ],
    'size': 32
  },
  'RemotingSnapshot': {
    'fields': [
      [
        'width',
        8,
        0,
        'f64'
      ],
      [
        'height',
        16,
        0,
        'f64'
      ],
      [
        'mimeType',
        24,
        0,
        'string'
      ],
      [
        'data',
        32,
        0,
        'bytes'
      ]
    ],
    'size': 40
  },
  'RemotingStartOptions': {
    'fields': [
      [
        'authKey',
        8,
        0,
        'string'
      ],
      [
        'iceServers',
        16,
        0,
        {
          'array': {
            'fields': [
              [
                'urls',
                8,
                0,
                {
                  'array': 'string',
                  'boolean': false,
                  'width': 8
                }
              ],
              [
                'username',
                16,
                0,
                { 'nullable': 'string' }
              ],
              [
                'credential',
                24,
                0,
                { 'nullable': 'string' }
              ]
            ],
            'size': 32
          },
          'boolean': false,
          'width': 8
        }
      ],
      [
        'maxBitrateKbps',
        32,
        0,
        { 'nullable': 'f64' },
        [24, 0]
      ],
      [
        'targetFramerate',
        40,
        0,
        { 'nullable': 'f64' },
        [24, 1]
      ],
      [
        'screenId',
        48,
        0,
        { 'nullable': 'i64' },
        [24, 2]
      ]
    ],
    'size': 56
  },
  'RemotingStartRequest': {
    'fields': [[
      'options',
      8,
      0,
      {
        'fields': [
          [
            'authKey',
            8,
            0,
            'string'
          ],
          [
            'iceServers',
            16,
            0,
            {
              'array': {
                'fields': [
                  [
                    'urls',
                    8,
                    0,
                    {
                      'array': 'string',
                      'boolean': false,
                      'width': 8
                    }
                  ],
                  [
                    'username',
                    16,
                    0,
                    { 'nullable': 'string' }
                  ],
                  [
                    'credential',
                    24,
                    0,
                    { 'nullable': 'string' }
                  ]
                ],
                'size': 32
              },
              'boolean': false,
              'width': 8
            }
          ],
          [
            'maxBitrateKbps',
            32,
            0,
            { 'nullable': 'f64' },
            [24, 0]
          ],
          [
            'targetFramerate',
            40,
            0,
            { 'nullable': 'f64' },
            [24, 1]
          ],
          [
            'screenId',
            48,
            0,
            { 'nullable': 'i64' },
            [24, 2]
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'RemotingStartResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [[
          'id',
          8,
          0,
          'i64'
        ], [
          'state',
          16,
          0,
          'string'
        ]],
        'size': 24
      }
    ]],
    'size': 16
  },
  'RemotingStats': {
    'fields': [
      [
        'state',
        8,
        0,
        'string'
      ],
      [
        'route',
        16,
        0,
        'string'
      ],
      [
        'transport',
        24,
        0,
        'string'
      ],
      [
        'latencyMs',
        32,
        0,
        'f64'
      ],
      [
        'jitterMs',
        40,
        0,
        'f64'
      ],
      [
        'availableBandwidthKbps',
        48,
        0,
        'f64'
      ],
      [
        'packetLossPercent',
        56,
        0,
        'f64'
      ]
    ],
    'size': 64
  },
  'RemotingStatsRequest': {
    'fields': [[
      'id',
      8,
      0,
      'i64'
    ]],
    'size': 16
  },
  'RemotingStatsResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'state',
            8,
            0,
            'string'
          ],
          [
            'route',
            16,
            0,
            'string'
          ],
          [
            'transport',
            24,
            0,
            'string'
          ],
          [
            'latencyMs',
            32,
            0,
            'f64'
          ],
          [
            'jitterMs',
            40,
            0,
            'f64'
          ],
          [
            'availableBandwidthKbps',
            48,
            0,
            'f64'
          ],
          [
            'packetLossPercent',
            56,
            0,
            'f64'
          ]
        ],
        'size': 64
      }
    ]],
    'size': 16
  },
  'RemotingStopRequest': {
    'fields': [[
      'id',
      8,
      0,
      'i64'
    ]],
    'size': 16
  },
  'RemotingStopResponse': {
    'fields': [],
    'size': 8
  },
  'RemotingTakeSignalsRequest': {
    'fields': [[
      'id',
      8,
      0,
      'i64'
    ], [
      'afterSequence',
      24,
      0,
      { 'nullable': 'i64' },
      [16, 0]
    ]],
    'size': 32
  },
  'RemotingTakeSignalsResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'array': {
          'fields': [
            [
              'sequence',
              8,
              0,
              'i64'
            ],
            [
              'description',
              16,
              0,
              { 'nullable': {
                'fields': [[
                  'type',
                  8,
                  0,
                  'string'
                ], [
                  'sdp',
                  16,
                  0,
                  'string'
                ]],
                'size': 24
              } }
            ],
            [
              'candidates',
              24,
              0,
              {
                'array': {
                  'fields': [
                    [
                      'mid',
                      8,
                      0,
                      'string'
                    ],
                    [
                      'mLineIndex',
                      16,
                      0,
                      'f64'
                    ],
                    [
                      'candidate',
                      24,
                      0,
                      'string'
                    ]
                  ],
                  'size': 32
                },
                'boolean': false,
                'width': 8
              }
            ]
          ],
          'size': 32
        },
        'boolean': false,
        'width': 8
      }
    ]],
    'size': 16
  },
  'StorageClearRequest': {
    'fields': [],
    'size': 8
  },
  'StorageClearResponse': {
    'fields': [],
    'size': 8
  },
  'StorageGetRequest': {
    'fields': [[
      'key',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'StorageGetResponse': {
    'fields': [[
      'result',
      8,
      0,
      { 'nullable': 'bytes' }
    ]],
    'size': 16
  },
  'StorageRemoveRequest': {
    'fields': [[
      'key',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'StorageRemoveResponse': {
    'fields': [],
    'size': 8
  },
  'StorageSetRequest': {
    'fields': [[
      'key',
      8,
      0,
      'string'
    ], [
      'value',
      16,
      0,
      'bytes'
    ]],
    'size': 24
  },
  'StorageSetResponse': {
    'fields': [],
    'size': 8
  },
  'UtilityCancelRequest': {
    'fields': [[
      'name',
      8,
      0,
      'string'
    ], [
      'requestId',
      16,
      0,
      'i64'
    ]],
    'size': 24
  },
  'UtilityCancelResponse': {
    'fields': [],
    'size': 8
  },
  'UtilityInvokeRequest': {
    'fields': [
      [
        'name',
        8,
        0,
        'string'
      ],
      [
        'requestId',
        16,
        0,
        'i64'
      ],
      [
        'payload',
        24,
        0,
        { 'nullable': 'bytes' }
      ]
    ],
    'size': 32
  },
  'UtilityInvokeResponse': {
    'fields': [[
      'result',
      8,
      0,
      'bytes'
    ]],
    'size': 16
  },
  'WindowBounds': {
    'fields': [
      [
        'x',
        8,
        0,
        'f64'
      ],
      [
        'y',
        16,
        0,
        'f64'
      ],
      [
        'width',
        24,
        0,
        'f64'
      ],
      [
        'height',
        32,
        0,
        'f64'
      ]
    ],
    'size': 40
  },
  'WindowCloseRequest': {
    'fields': [[
      'id',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 0]
    ]],
    'size': 24
  },
  'WindowCloseResponse': {
    'fields': [],
    'size': 8
  },
  'WindowCreateRequest': {
    'fields': [[
      'options',
      8,
      0,
      { 'nullable': {
        'fields': [
          [
            'label',
            8,
            0,
            { 'nullable': 'string' }
          ],
          [
            'url',
            16,
            0,
            { 'nullable': 'string' }
          ],
          [
            'title',
            24,
            0,
            { 'nullable': 'string' }
          ],
          [
            'bounds',
            32,
            0,
            { 'nullable': {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            } }
          ],
          [
            'focused',
            40,
            1,
            { 'nullable': 'bool' },
            [40, 0]
          ]
        ],
        'size': 48
      } }
    ]],
    'size': 16
  },
  'WindowCreateResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowCurrentRequest': {
    'fields': [],
    'size': 8
  },
  'WindowCurrentResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowFocusRequest': {
    'fields': [[
      'id',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 0]
    ]],
    'size': 24
  },
  'WindowFocusResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowGetBoundsRequest': {
    'fields': [[
      'id',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 0]
    ]],
    'size': 24
  },
  'WindowGetBoundsResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'x',
            8,
            0,
            'f64'
          ],
          [
            'y',
            16,
            0,
            'f64'
          ],
          [
            'width',
            24,
            0,
            'f64'
          ],
          [
            'height',
            32,
            0,
            'f64'
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'WindowInfo': {
    'fields': [
      [
        'id',
        8,
        0,
        'i64'
      ],
      [
        'label',
        16,
        0,
        'string'
      ],
      [
        'title',
        24,
        0,
        'string'
      ],
      [
        'url',
        32,
        0,
        'string'
      ],
      [
        'bounds',
        40,
        0,
        {
          'fields': [
            [
              'x',
              8,
              0,
              'f64'
            ],
            [
              'y',
              16,
              0,
              'f64'
            ],
            [
              'width',
              24,
              0,
              'f64'
            ],
            [
              'height',
              32,
              0,
              'f64'
            ]
          ],
          'size': 40
        }
      ],
      [
        'focused',
        48,
        0,
        'bool'
      ],
      [
        'minimized',
        48,
        1,
        'bool'
      ],
      [
        'maximized',
        48,
        2,
        'bool'
      ],
      [
        'fullscreen',
        48,
        3,
        'bool'
      ]
    ],
    'size': 56
  },
  'WindowListRequest': {
    'fields': [],
    'size': 8
  },
  'WindowListResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'array': {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        },
        'boolean': false,
        'width': 8
      }
    ]],
    'size': 16
  },
  'WindowMaximizeRequest': {
    'fields': [[
      'id',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 0]
    ]],
    'size': 24
  },
  'WindowMaximizeResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowMinimizeRequest': {
    'fields': [[
      'id',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 0]
    ]],
    'size': 24
  },
  'WindowMinimizeResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowOptions': {
    'fields': [
      [
        'label',
        8,
        0,
        { 'nullable': 'string' }
      ],
      [
        'url',
        16,
        0,
        { 'nullable': 'string' }
      ],
      [
        'title',
        24,
        0,
        { 'nullable': 'string' }
      ],
      [
        'bounds',
        32,
        0,
        { 'nullable': {
          'fields': [
            [
              'x',
              8,
              0,
              'f64'
            ],
            [
              'y',
              16,
              0,
              'f64'
            ],
            [
              'width',
              24,
              0,
              'f64'
            ],
            [
              'height',
              32,
              0,
              'f64'
            ]
          ],
          'size': 40
        } }
      ],
      [
        'focused',
        40,
        1,
        { 'nullable': 'bool' },
        [40, 0]
      ]
    ],
    'size': 48
  },
  'WindowRestoreRequest': {
    'fields': [[
      'id',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 0]
    ]],
    'size': 24
  },
  'WindowRestoreResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowSetBoundsRequest': {
    'fields': [[
      'bounds',
      8,
      0,
      {
        'fields': [
          [
            'x',
            8,
            0,
            'f64'
          ],
          [
            'y',
            16,
            0,
            'f64'
          ],
          [
            'width',
            24,
            0,
            'f64'
          ],
          [
            'height',
            32,
            0,
            'f64'
          ]
        ],
        'size': 40
      }
    ], [
      'id',
      24,
      0,
      { 'nullable': 'i64' },
      [16, 0]
    ]],
    'size': 32
  },
  'WindowSetBoundsResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowSetFullscreenRequest': {
    'fields': [[
      'fullscreen',
      8,
      0,
      'bool'
    ], [
      'id',
      16,
      0,
      { 'nullable': 'i64' },
      [8, 1]
    ]],
    'size': 24
  },
  'WindowSetFullscreenResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  },
  'WindowSetTitleRequest': {
    'fields': [[
      'title',
      8,
      0,
      'string'
    ], [
      'id',
      24,
      0,
      { 'nullable': 'i64' },
      [16, 0]
    ]],
    'size': 32
  },
  'WindowSetTitleResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'id',
            8,
            0,
            'i64'
          ],
          [
            'label',
            16,
            0,
            'string'
          ],
          [
            'title',
            24,
            0,
            'string'
          ],
          [
            'url',
            32,
            0,
            'string'
          ],
          [
            'bounds',
            40,
            0,
            {
              'fields': [
                [
                  'x',
                  8,
                  0,
                  'f64'
                ],
                [
                  'y',
                  16,
                  0,
                  'f64'
                ],
                [
                  'width',
                  24,
                  0,
                  'f64'
                ],
                [
                  'height',
                  32,
                  0,
                  'f64'
                ]
              ],
              'size': 40
            }
          ],
          [
            'focused',
            48,
            0,
            'bool'
          ],
          [
            'minimized',
            48,
            1,
            'bool'
          ],
          [
            'maximized',
            48,
            2,
            'bool'
          ],
          [
            'fullscreen',
            48,
            3,
            'bool'
          ]
        ],
        'size': 56
      }
    ]],
    'size': 16
  }
};
export const mojomMethods = [
  [
    'dialog.openFile',
    '0',
    '0',
    {
      'fields': [[
        'options',
        8,
        0,
        { 'nullable': {
          'fields': [
            [
              'title',
              8,
              0,
              { 'nullable': 'string' }
            ],
            [
              'defaultPath',
              16,
              0,
              { 'nullable': 'string' }
            ],
            [
              'extensions',
              24,
              0,
              { 'nullable': {
                'array': {
                  'array': 'string',
                  'boolean': false,
                  'width': 8
                },
                'boolean': false,
                'width': 8
              } }
            ]
          ],
          'size': 32
        } }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        { 'nullable': 'string' }
      ]],
      'size': 16
    },
    'tuffite.Dialog.OpenFile'
  ],
  [
    'dialog.openDirectory',
    '0',
    '0',
    {
      'fields': [[
        'options',
        8,
        0,
        { 'nullable': {
          'fields': [[
            'title',
            8,
            0,
            { 'nullable': 'string' }
          ]],
          'size': 16
        } }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        { 'nullable': 'string' }
      ]],
      'size': 16
    },
    'tuffite.Dialog.OpenDirectory'
  ],
  [
    'path.directories',
    '0',
    '0',
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'resource',
              8,
              0,
              'string'
            ],
            [
              'data',
              16,
              0,
              'string'
            ],
            [
              'cache',
              24,
              0,
              'string'
            ],
            [
              'temp',
              32,
              0,
              'string'
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    'tuffite.Path.Directories'
  ],
  [
    'path.resolve',
    '0',
    '0',
    {
      'fields': [[
        'base',
        8,
        0,
        { 'enum': [
          0,
          1,
          2,
          3
        ] }
      ], [
        'relative',
        16,
        0,
        { 'nullable': 'string' }
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'tuffite.Path.Resolve'
  ],
  [
    'remoting.capabilities',
    '0',
    '0',
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'available',
              8,
              0,
              'bool'
            ],
            [
              'protocol',
              16,
              0,
              'string'
            ],
            [
              'videoCodecs',
              24,
              0,
              {
                'array': 'string',
                'boolean': false,
                'width': 8
              }
            ],
            [
              'input',
              8,
              1,
              'bool'
            ],
            [
              'clipboard',
              8,
              2,
              'bool'
            ],
            [
              'reconnect',
              8,
              3,
              'bool'
            ],
            [
              'snapshot',
              8,
              4,
              'bool'
            ]
          ],
          'size': 32
        }
      ]],
      'size': 16
    },
    'tuffite.Remoting.Capabilities'
  ],
  [
    'remoting.start',
    '0',
    '0',
    {
      'fields': [[
        'options',
        8,
        0,
        {
          'fields': [
            [
              'authKey',
              8,
              0,
              'string'
            ],
            [
              'iceServers',
              16,
              0,
              {
                'array': {
                  'fields': [
                    [
                      'urls',
                      8,
                      0,
                      {
                        'array': 'string',
                        'boolean': false,
                        'width': 8
                      }
                    ],
                    [
                      'username',
                      16,
                      0,
                      { 'nullable': 'string' }
                    ],
                    [
                      'credential',
                      24,
                      0,
                      { 'nullable': 'string' }
                    ]
                  ],
                  'size': 32
                },
                'boolean': false,
                'width': 8
              }
            ],
            [
              'maxBitrateKbps',
              32,
              0,
              { 'nullable': 'f64' },
              [24, 0]
            ],
            [
              'targetFramerate',
              40,
              0,
              { 'nullable': 'f64' },
              [24, 1]
            ],
            [
              'screenId',
              48,
              0,
              { 'nullable': 'i64' },
              [24, 2]
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [[
            'id',
            8,
            0,
            'i64'
          ], [
            'state',
            16,
            0,
            'string'
          ]],
          'size': 24
        }
      ]],
      'size': 16
    },
    'tuffite.Remoting.Start'
  ],
  [
    'remoting.processSignal',
    '0',
    '0',
    {
      'fields': [[
        'id',
        8,
        0,
        'i64'
      ], [
        'signal',
        16,
        0,
        {
          'fields': [
            [
              'sequence',
              8,
              0,
              'i64'
            ],
            [
              'description',
              16,
              0,
              { 'nullable': {
                'fields': [[
                  'type',
                  8,
                  0,
                  'string'
                ], [
                  'sdp',
                  16,
                  0,
                  'string'
                ]],
                'size': 24
              } }
            ],
            [
              'candidates',
              24,
              0,
              {
                'array': {
                  'fields': [
                    [
                      'mid',
                      8,
                      0,
                      'string'
                    ],
                    [
                      'mLineIndex',
                      16,
                      0,
                      'f64'
                    ],
                    [
                      'candidate',
                      24,
                      0,
                      'string'
                    ]
                  ],
                  'size': 32
                },
                'boolean': false,
                'width': 8
              }
            ]
          ],
          'size': 32
        }
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Remoting.ProcessSignal'
  ],
  [
    'remoting.takeSignals',
    '0',
    '0',
    {
      'fields': [[
        'id',
        8,
        0,
        'i64'
      ], [
        'afterSequence',
        24,
        0,
        { 'nullable': 'i64' },
        [16, 0]
      ]],
      'size': 32
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'array': {
            'fields': [
              [
                'sequence',
                8,
                0,
                'i64'
              ],
              [
                'description',
                16,
                0,
                { 'nullable': {
                  'fields': [[
                    'type',
                    8,
                    0,
                    'string'
                  ], [
                    'sdp',
                    16,
                    0,
                    'string'
                  ]],
                  'size': 24
                } }
              ],
              [
                'candidates',
                24,
                0,
                {
                  'array': {
                    'fields': [
                      [
                        'mid',
                        8,
                        0,
                        'string'
                      ],
                      [
                        'mLineIndex',
                        16,
                        0,
                        'f64'
                      ],
                      [
                        'candidate',
                        24,
                        0,
                        'string'
                      ]
                    ],
                    'size': 32
                  },
                  'boolean': false,
                  'width': 8
                }
              ]
            ],
            'size': 32
          },
          'boolean': false,
          'width': 8
        }
      ]],
      'size': 16
    },
    'tuffite.Remoting.TakeSignals'
  ],
  [
    'remoting.sendInput',
    '0',
    '0',
    {
      'fields': [[
        'id',
        8,
        0,
        'i64'
      ], [
        'event',
        16,
        0,
        {
          'fields': [
            [
              'kind',
              8,
              0,
              'string'
            ],
            [
              'pressed',
              16,
              1,
              { 'nullable': 'bool' },
              [16, 0]
            ],
            [
              'usbKeycode',
              24,
              0,
              { 'nullable': 'f64' },
              [16, 2]
            ],
            [
              'text',
              32,
              0,
              { 'nullable': 'string' }
            ],
            [
              'x',
              40,
              0,
              { 'nullable': 'f64' },
              [16, 3]
            ],
            [
              'y',
              48,
              0,
              { 'nullable': 'f64' },
              [16, 4]
            ],
            [
              'button',
              56,
              0,
              { 'nullable': 'string' }
            ],
            [
              'buttonDown',
              16,
              6,
              { 'nullable': 'bool' },
              [16, 5]
            ],
            [
              'wheelDeltaX',
              64,
              0,
              { 'nullable': 'f64' },
              [16, 7]
            ],
            [
              'wheelDeltaY',
              72,
              0,
              { 'nullable': 'f64' },
              [17, 0]
            ],
            [
              'wheelTicksX',
              80,
              0,
              { 'nullable': 'f64' },
              [17, 1]
            ],
            [
              'wheelTicksY',
              88,
              0,
              { 'nullable': 'f64' },
              [17, 2]
            ],
            [
              'clipboardMimeType',
              96,
              0,
              { 'nullable': 'string' }
            ],
            [
              'clipboardData',
              104,
              0,
              { 'nullable': 'bytes' }
            ]
          ],
          'size': 112
        }
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Remoting.SendInput'
  ],
  [
    'remoting.stop',
    '0',
    '0',
    {
      'fields': [[
        'id',
        8,
        0,
        'i64'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Remoting.Stop'
  ],
  [
    'remoting.stats',
    '0',
    '0',
    {
      'fields': [[
        'id',
        8,
        0,
        'i64'
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'state',
              8,
              0,
              'string'
            ],
            [
              'route',
              16,
              0,
              'string'
            ],
            [
              'transport',
              24,
              0,
              'string'
            ],
            [
              'latencyMs',
              32,
              0,
              'f64'
            ],
            [
              'jitterMs',
              40,
              0,
              'f64'
            ],
            [
              'availableBandwidthKbps',
              48,
              0,
              'f64'
            ],
            [
              'packetLossPercent',
              56,
              0,
              'f64'
            ]
          ],
          'size': 64
        }
      ]],
      'size': 16
    },
    'tuffite.Remoting.Stats'
  ],
  [
    'remoting.capture',
    '0',
    '0',
    {
      'fields': [[
        'screenId',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 0]
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'width',
              8,
              0,
              'f64'
            ],
            [
              'height',
              16,
              0,
              'f64'
            ],
            [
              'mimeType',
              24,
              0,
              'string'
            ],
            [
              'data',
              32,
              0,
              'bytes'
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    'tuffite.Remoting.Capture'
  ],
  [
    'remoting.injectInput',
    '0',
    '0',
    {
      'fields': [[
        'event',
        8,
        0,
        {
          'fields': [
            [
              'kind',
              8,
              0,
              'string'
            ],
            [
              'pressed',
              16,
              1,
              { 'nullable': 'bool' },
              [16, 0]
            ],
            [
              'usbKeycode',
              24,
              0,
              { 'nullable': 'f64' },
              [16, 2]
            ],
            [
              'text',
              32,
              0,
              { 'nullable': 'string' }
            ],
            [
              'x',
              40,
              0,
              { 'nullable': 'f64' },
              [16, 3]
            ],
            [
              'y',
              48,
              0,
              { 'nullable': 'f64' },
              [16, 4]
            ],
            [
              'button',
              56,
              0,
              { 'nullable': 'string' }
            ],
            [
              'buttonDown',
              16,
              6,
              { 'nullable': 'bool' },
              [16, 5]
            ],
            [
              'wheelDeltaX',
              64,
              0,
              { 'nullable': 'f64' },
              [16, 7]
            ],
            [
              'wheelDeltaY',
              72,
              0,
              { 'nullable': 'f64' },
              [17, 0]
            ],
            [
              'wheelTicksX',
              80,
              0,
              { 'nullable': 'f64' },
              [17, 1]
            ],
            [
              'wheelTicksY',
              88,
              0,
              { 'nullable': 'f64' },
              [17, 2]
            ],
            [
              'clipboardMimeType',
              96,
              0,
              { 'nullable': 'string' }
            ],
            [
              'clipboardData',
              104,
              0,
              { 'nullable': 'bytes' }
            ]
          ],
          'size': 112
        }
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Remoting.InjectInput'
  ],
  [
    'fs.readFile',
    '0',
    '0',
    {
      'fields': [[
        'base',
        8,
        0,
        { 'enum': [
          0,
          1,
          2,
          3
        ] }
      ], [
        'path',
        16,
        0,
        'string'
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'bytes'
      ]],
      'size': 16
    },
    'tuffite.Fs.ReadFile'
  ],
  [
    'fs.writeFile',
    '0',
    '0',
    {
      'fields': [
        [
          'base',
          8,
          0,
          { 'enum': [
            0,
            1,
            2,
            3
          ] }
        ],
        [
          'path',
          16,
          0,
          'string'
        ],
        [
          'contents',
          24,
          0,
          'bytes'
        ]
      ],
      'size': 32
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Fs.WriteFile'
  ],
  [
    'fs.exists',
    '0',
    '0',
    {
      'fields': [[
        'base',
        8,
        0,
        { 'enum': [
          0,
          1,
          2,
          3
        ] }
      ], [
        'path',
        16,
        0,
        'string'
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'bool'
      ]],
      'size': 16
    },
    'tuffite.Fs.Exists'
  ],
  [
    'fs.createDir',
    '0',
    '0',
    {
      'fields': [[
        'base',
        8,
        0,
        { 'enum': [
          0,
          1,
          2,
          3
        ] }
      ], [
        'path',
        16,
        0,
        'string'
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Fs.CreateDir'
  ],
  [
    'fs.removeFile',
    '0',
    '0',
    {
      'fields': [[
        'base',
        8,
        0,
        { 'enum': [
          0,
          1,
          2,
          3
        ] }
      ], [
        'path',
        16,
        0,
        'string'
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Fs.RemoveFile'
  ],
  [
    'storage.get',
    '0',
    '0',
    {
      'fields': [[
        'key',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        { 'nullable': 'bytes' }
      ]],
      'size': 16
    },
    'tuffite.Storage.Get'
  ],
  [
    'storage.set',
    '0',
    '0',
    {
      'fields': [[
        'key',
        8,
        0,
        'string'
      ], [
        'value',
        16,
        0,
        'bytes'
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Storage.Set'
  ],
  [
    'storage.remove',
    '0',
    '0',
    {
      'fields': [[
        'key',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Storage.Remove'
  ],
  [
    'storage.clear',
    '0',
    '0',
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Storage.Clear'
  ],
  [
    'log.trace',
    '0',
    '0',
    {
      'fields': [[
        'message',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Log.Trace'
  ],
  [
    'log.debug',
    '0',
    '0',
    {
      'fields': [[
        'message',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Log.Debug'
  ],
  [
    'log.info',
    '0',
    '0',
    {
      'fields': [[
        'message',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Log.Info'
  ],
  [
    'log.warn',
    '0',
    '0',
    {
      'fields': [[
        'message',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Log.Warn'
  ],
  [
    'log.error',
    '0',
    '0',
    {
      'fields': [[
        'message',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Log.Error'
  ],
  [
    'net.request',
    '0',
    '0',
    {
      'fields': [[
        'url',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [[
            'status',
            8,
            0,
            'f64'
          ], [
            'body',
            16,
            0,
            'bytes'
          ]],
          'size': 24
        }
      ]],
      'size': 16
    },
    'tuffite.Net.Request'
  ],
  [
    'utility.invoke',
    '0',
    '0',
    {
      'fields': [
        [
          'name',
          8,
          0,
          'string'
        ],
        [
          'requestId',
          16,
          0,
          'i64'
        ],
        [
          'payload',
          24,
          0,
          { 'nullable': 'bytes' }
        ]
      ],
      'size': 32
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'bytes'
      ]],
      'size': 16
    },
    'tuffite.Utility.Invoke'
  ],
  [
    'utility.cancel',
    '0',
    '0',
    {
      'fields': [[
        'name',
        8,
        0,
        'string'
      ], [
        'requestId',
        16,
        0,
        'i64'
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Utility.Cancel'
  ],
  [
    'window.current',
    '0',
    '0',
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.Current'
  ],
  [
    'window.list',
    '0',
    '0',
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'array': {
            'fields': [
              [
                'id',
                8,
                0,
                'i64'
              ],
              [
                'label',
                16,
                0,
                'string'
              ],
              [
                'title',
                24,
                0,
                'string'
              ],
              [
                'url',
                32,
                0,
                'string'
              ],
              [
                'bounds',
                40,
                0,
                {
                  'fields': [
                    [
                      'x',
                      8,
                      0,
                      'f64'
                    ],
                    [
                      'y',
                      16,
                      0,
                      'f64'
                    ],
                    [
                      'width',
                      24,
                      0,
                      'f64'
                    ],
                    [
                      'height',
                      32,
                      0,
                      'f64'
                    ]
                  ],
                  'size': 40
                }
              ],
              [
                'focused',
                48,
                0,
                'bool'
              ],
              [
                'minimized',
                48,
                1,
                'bool'
              ],
              [
                'maximized',
                48,
                2,
                'bool'
              ],
              [
                'fullscreen',
                48,
                3,
                'bool'
              ]
            ],
            'size': 56
          },
          'boolean': false,
          'width': 8
        }
      ]],
      'size': 16
    },
    'tuffite.Window.List'
  ],
  [
    'window.create',
    '0',
    '0',
    {
      'fields': [[
        'options',
        8,
        0,
        { 'nullable': {
          'fields': [
            [
              'label',
              8,
              0,
              { 'nullable': 'string' }
            ],
            [
              'url',
              16,
              0,
              { 'nullable': 'string' }
            ],
            [
              'title',
              24,
              0,
              { 'nullable': 'string' }
            ],
            [
              'bounds',
              32,
              0,
              { 'nullable': {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              } }
            ],
            [
              'focused',
              40,
              1,
              { 'nullable': 'bool' },
              [40, 0]
            ]
          ],
          'size': 48
        } }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.Create'
  ],
  [
    'window.focus',
    '0',
    '0',
    {
      'fields': [[
        'id',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 0]
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.Focus'
  ],
  [
    'window.minimize',
    '0',
    '0',
    {
      'fields': [[
        'id',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 0]
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.Minimize'
  ],
  [
    'window.maximize',
    '0',
    '0',
    {
      'fields': [[
        'id',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 0]
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.Maximize'
  ],
  [
    'window.restore',
    '0',
    '0',
    {
      'fields': [[
        'id',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 0]
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.Restore'
  ],
  [
    'window.setFullscreen',
    '0',
    '0',
    {
      'fields': [[
        'fullscreen',
        8,
        0,
        'bool'
      ], [
        'id',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 1]
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.SetFullscreen'
  ],
  [
    'window.close',
    '0',
    '0',
    {
      'fields': [[
        'id',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 0]
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.Window.Close'
  ],
  [
    'window.setTitle',
    '0',
    '0',
    {
      'fields': [[
        'title',
        8,
        0,
        'string'
      ], [
        'id',
        24,
        0,
        { 'nullable': 'i64' },
        [16, 0]
      ]],
      'size': 32
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.SetTitle'
  ],
  [
    'window.getBounds',
    '0',
    '0',
    {
      'fields': [[
        'id',
        16,
        0,
        { 'nullable': 'i64' },
        [8, 0]
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'x',
              8,
              0,
              'f64'
            ],
            [
              'y',
              16,
              0,
              'f64'
            ],
            [
              'width',
              24,
              0,
              'f64'
            ],
            [
              'height',
              32,
              0,
              'f64'
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    'tuffite.Window.GetBounds'
  ],
  [
    'window.setBounds',
    '0',
    '0',
    {
      'fields': [[
        'bounds',
        8,
        0,
        {
          'fields': [
            [
              'x',
              8,
              0,
              'f64'
            ],
            [
              'y',
              16,
              0,
              'f64'
            ],
            [
              'width',
              24,
              0,
              'f64'
            ],
            [
              'height',
              32,
              0,
              'f64'
            ]
          ],
          'size': 40
        }
      ], [
        'id',
        24,
        0,
        { 'nullable': 'i64' },
        [16, 0]
      ]],
      'size': 32
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'id',
              8,
              0,
              'i64'
            ],
            [
              'label',
              16,
              0,
              'string'
            ],
            [
              'title',
              24,
              0,
              'string'
            ],
            [
              'url',
              32,
              0,
              'string'
            ],
            [
              'bounds',
              40,
              0,
              {
                'fields': [
                  [
                    'x',
                    8,
                    0,
                    'f64'
                  ],
                  [
                    'y',
                    16,
                    0,
                    'f64'
                  ],
                  [
                    'width',
                    24,
                    0,
                    'f64'
                  ],
                  [
                    'height',
                    32,
                    0,
                    'f64'
                  ]
                ],
                'size': 40
              }
            ],
            [
              'focused',
              48,
              0,
              'bool'
            ],
            [
              'minimized',
              48,
              1,
              'bool'
            ],
            [
              'maximized',
              48,
              2,
              'bool'
            ],
            [
              'fullscreen',
              48,
              3,
              'bool'
            ]
          ],
          'size': 56
        }
      ]],
      'size': 16
    },
    'tuffite.Window.SetBounds'
  ]
];
export const BaseDirectory = Object.freeze({
  '0': 'Resource',
  '1': 'Data',
  '2': 'Cache',
  '3': 'Temp',
  'Cache': 2,
  'Data': 1,
  'Resource': 0,
  'Temp': 3
});
export function createTuffiteApi(transport = invokeFrameworkMojom) {
  return createMojomCommandApi(mojomMethods, 'tuffite', transport);
}
export const { dialog, fs, log, net, path, remoting, storage, utility, window } = createTuffiteApi();

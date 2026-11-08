// Includes Chromium Mojo bindings (BSD-3-Clause); see tuffite/CHROMIUM_LICENSE.
// Generated from Mojom v0. Do not edit.
const mojo = {
  internal: { interfaceSupport: {} },
  interfaceControl: {},
  pipeControl: {}
};
mojo.internal.kArrayHeaderSize = 8, mojo.internal.kStructHeaderSize = 8, mojo.internal.kStructHeaderSizeOffset = 0, mojo.internal.kStructHeaderVersionOffset = 4, mojo.internal.kUnionHeaderSize = 8, mojo.internal.kUnionDataSize = 16, mojo.internal.kMessageV0HeaderSize = 24, mojo.internal.kMessageV1HeaderSize = 32, mojo.internal.kMessageV2HeaderSize = 48, mojo.internal.kMapDataSize = 24, mojo.internal.kEncodedInvalidHandleValue = 4294967295, mojo.internal.kMessageFlagExpectsResponse = 1, mojo.internal.kMessageFlagIsResponse = 2, mojo.internal.kInterfaceNamespaceBit = 2147483648, mojo.internal.kHostLittleEndian = !!new Uint8Array(new Uint16Array([1]).buffer)[0], mojo.internal.DataViewWrapper = class {
  constructor(e, n = 0) {
    if (this.dataView_, this.byteOffset, e instanceof mojo.internal.DataViewWrapper) this.dataView_ = e.dataView_, this.byteOffset = e.byteOffset + n;
    else if (e instanceof DataView) this.dataView_ = 0 === e.byteOffset && e.byteLength === e.buffer.byteLength ? e : new DataView(e.buffer), this.byteOffset = e.byteOffset + n;
    else {
      if (!(e instanceof ArrayBuffer)) throw new Error('DataViewWrapper constructor requires a DataViewWrapper, DataView, or ArrayBuffer instance');
      this.dataView_ = new DataView(e), this.byteOffset = n;
    }
  }
  get buffer() {
    return this.dataView_.buffer;
  }
  getUint8(e) {
    return this.dataView_.getUint8(this.byteOffset + e);
  }
  getInt8(e) {
    return this.dataView_.getInt8(this.byteOffset + e);
  }
  getUint16(e, n) {
    return this.dataView_.getUint16(this.byteOffset + e, n);
  }
  getInt16(e, n) {
    return this.dataView_.getInt16(this.byteOffset + e, n);
  }
  getUint32(e, n) {
    return this.dataView_.getUint32(this.byteOffset + e, n);
  }
  getInt32(e, n) {
    return this.dataView_.getInt32(this.byteOffset + e, n);
  }
  getFloat32(e, n) {
    return this.dataView_.getFloat32(this.byteOffset + e, n);
  }
  getFloat64(e, n) {
    return this.dataView_.getFloat64(this.byteOffset + e, n);
  }
  setUint8(e, n) {
    this.dataView_.setUint8(this.byteOffset + e, n);
  }
  setInt8(e, n) {
    this.dataView_.setInt8(this.byteOffset + e, n);
  }
  setUint16(e, n, t) {
    this.dataView_.setUint16(this.byteOffset + e, n, t);
  }
  setInt16(e, n, t) {
    this.dataView_.setInt16(this.byteOffset + e, n, t);
  }
  setUint32(e, n, t) {
    this.dataView_.setUint32(this.byteOffset + e, n, t);
  }
  setInt32(e, n, t) {
    this.dataView_.setInt32(this.byteOffset + e, n, t);
  }
  setFloat32(e, n, t) {
    this.dataView_.setFloat32(this.byteOffset + e, n, t);
  }
  setFloat64(e, n, t) {
    this.dataView_.setFloat64(this.byteOffset + e, n, t);
  }
}, mojo.internal.isNullOrUndefined = function(e) {
  return null == e;
}, mojo.internal.isNullableValueKindField = function(e) {
  return void 0 !== e.nullableValueKindProperties;
}, mojo.internal.align = function(e, n) {
  return e + (n - e % n) % n;
}, mojo.internal.setInt64 = function(e, n, t) {
  mojo.internal.kHostLittleEndian ? (e.setUint32(n, Number(BigInt(t) & BigInt(4294967295)), mojo.internal.kHostLittleEndian), e.setInt32(n + 4, Number(BigInt(t) >> BigInt(32) & BigInt(4294967295)), mojo.internal.kHostLittleEndian)) : (e.setInt32(n, Number(BigInt(t) >> BigInt(32) & BigInt(4294967295)), mojo.internal.kHostLittleEndian), e.setUint32(n + 4, Number(BigInt(t) & BigInt(4294967295)), mojo.internal.kHostLittleEndian));
}, mojo.internal.setUint64 = function(e, n, t) {
  mojo.internal.kHostLittleEndian ? (e.setUint32(n, Number(BigInt(t) & BigInt(4294967295)), mojo.internal.kHostLittleEndian), e.setUint32(n + 4, Number(BigInt(t) >> BigInt(32) & BigInt(4294967295)), mojo.internal.kHostLittleEndian)) : (e.setUint32(n, Number(BigInt(t) >> BigInt(32) & BigInt(4294967295)), mojo.internal.kHostLittleEndian), e.setUint32(n + 4, Number(BigInt(t) & BigInt(4294967295)), mojo.internal.kHostLittleEndian));
}, mojo.internal.getInt64 = function(e, n) {
  let t, o;
  return mojo.internal.kHostLittleEndian ? (t = e.getUint32(n, mojo.internal.kHostLittleEndian), o = e.getInt32(n + 4, mojo.internal.kHostLittleEndian)) : (t = e.getUint32(n + 4, mojo.internal.kHostLittleEndian), o = e.getInt32(n, mojo.internal.kHostLittleEndian)), BigInt(o) << BigInt(32) | BigInt(t);
}, mojo.internal.getUint64 = function(e, n) {
  let t, o;
  return mojo.internal.kHostLittleEndian ? (t = e.getUint32(n, mojo.internal.kHostLittleEndian), o = e.getUint32(n + 4, mojo.internal.kHostLittleEndian)) : (t = e.getUint32(n + 4, mojo.internal.kHostLittleEndian), o = e.getUint32(n, mojo.internal.kHostLittleEndian)), BigInt(o) << BigInt(32) | BigInt(t);
}, mojo.internal.MessageDimensions, mojo.internal.getMojoFieldValue = function(e, n) {
  if (n.fieldGetter) return n.fieldGetter(e);
  if (e && mojo.internal.isNullableValueKindField(n)) {
    const t = n.nullableValueKindProperties, o = !mojo.internal.isNullOrUndefined(e[t.originalFieldName]);
    return t.isPrimary ? o : o ? e[t.originalFieldName] : n.defaultValue;
  }
  return e[n.name];
}, mojo.internal.computeStructDimensions = function(e, n) {
  let t = e.packedSize, o = 0;
  for (const i of e.fields) {
    let e = mojo.internal.getMojoFieldValue(n, i);
    if (mojo.internal.isNullOrUndefined(e) && (e = i.defaultValue), null !== e) if (i.type.$.computeDimensions) {
      const n = i.type.$.computeDimensions(e, i.nullable);
      t += mojo.internal.align(n.size, 8), n.numInterfaceIds && (o += n.numInterfaceIds);
    } else i.type.$.hasInterfaceId && o++;
  }
  return {
    size: t,
    numInterfaceIds: o
  };
}, mojo.internal.computeUnionDimensions = function(e, n, t) {
  let o = n ? mojo.internal.kUnionDataSize : 0, i = 0;
  const r = Object.keys(t);
  if (1 !== r.length) throw new Error(`Value for ${e.name} must be an Object with a single property named one of: ` + Object.keys(e.fields).join(','));
  const s = r[0], a = e.fields[s], l = t[s];
  if (!mojo.internal.isNullOrUndefined(l)) if (a.type.$.computeDimensions) {
    const e = !!a.type.$.unionSpec || a.nullable, n = a.type.$.computeDimensions(l, e);
    o += mojo.internal.align(n.size, 8), n.numInterfaceIds && (i += n.numInterfaceIds);
  } else a.type.$.hasInterfaceId && i++;
  return {
    size: o,
    numInterfaceIds: i
  };
}, mojo.internal.computeInlineArraySize = function(e, n) {
  return e.elementType === mojo.internal.Bool ? mojo.internal.kArrayHeaderSize + mojo.internal.computeHasValueBitfieldSize(e, n.length) + (n.length + 7 >> 3) : mojo.internal.kArrayHeaderSize + mojo.internal.computeHasValueBitfieldSize(e, n.length) + n.length * e.elementType.$.arrayElementSize(!!e.elementNullable);
}, mojo.internal.computeHasValueBitfieldSize = function(e, n) {
  if (!(!!e.elementNullable && !!e.elementType.$.isValueType)) return 0;
  const t = e.elementType.$.arrayElementSize(!0), o = 8 * t;
  return ((n + o - 1) / o | 0) * t;
}, mojo.internal.computeTotalArraySize = function(e, n) {
  const t = mojo.internal.computeInlineArraySize(e, n);
  if (!e.elementType.$.computeDimensions) return t;
  let o = t;
  for (let t of n) mojo.internal.isNullOrUndefined(t) || (o += mojo.internal.align(e.elementType.$.computeDimensions(t, !!e.elementNullable).size, 8));
  return o;
}, mojo.internal.Message = class {
  constructor(e, n, t, o, i, r, s) {
    const a = mojo.internal.computeStructDimensions(r, s);
    let l, c;
    a.numInterfaceIds > 0 ? (l = mojo.internal.kMessageV2HeaderSize, c = 2) : t & (mojo.internal.kMessageFlagExpectsResponse | mojo.internal.kMessageFlagIsResponse) ? (l = mojo.internal.kMessageV1HeaderSize, c = 1) : (l = mojo.internal.kMessageV0HeaderSize, c = 0);
    const d = l + a.size, u = a.numInterfaceIds > 0 ? mojo.internal.kArrayHeaderSize + 4 * a.numInterfaceIds : 0, p = d + mojo.internal.align(u, 8);
    this.buffer = new ArrayBuffer(p), this.handles = [];
    const m = new DataView(this.buffer);
    m.setUint32(0, l, mojo.internal.kHostLittleEndian), m.setUint32(4, c, mojo.internal.kHostLittleEndian), m.setUint32(8, n, mojo.internal.kHostLittleEndian), m.setUint32(12, o, mojo.internal.kHostLittleEndian), m.setUint32(16, t, mojo.internal.kHostLittleEndian), m.setUint32(20, 0), c >= 1 && (mojo.internal.setUint64(m, 24, i), c >= 2 && (mojo.internal.setUint64(m, 32, BigInt(16)), mojo.internal.setUint64(m, 40, BigInt(d - 40)), m.setUint32(d, u, mojo.internal.kHostLittleEndian), m.setUint32(d + 4, a.numInterfaceIds || 0, mojo.internal.kHostLittleEndian))), this.nextInterfaceIdIndex_ = 0, this.interfaceIds_ = null, a.numInterfaceIds && (this.interfaceIds_ = new Uint32Array(this.buffer, d + mojo.internal.kArrayHeaderSize, a.numInterfaceIds)), this.nextAllocationOffset_ = l;
    const f = this.allocate(r.packedSize);
    new mojo.internal.Encoder(this, f, { endpoint: e }).encodeStructInline(r, s);
  }
  allocate(e) {
    const n = mojo.internal.align(e, 8), t = new DataView(this.buffer, this.nextAllocationOffset_, n);
    return this.nextAllocationOffset_ += n, t;
  }
}, mojo.internal.MessageContext, mojo.internal.Encoder = class {
  constructor(e, n, t = null) {
    this.context_ = t, this.message_ = e, this.data_ = n;
  }
  encodeBool(e, n, t) {
    const o = this.data_.getUint8(e);
    t ? this.data_.setUint8(e, o | 1 << n) : this.data_.setUint8(e, o & ~(1 << n));
  }
  encodeInt8(e, n) {
    this.data_.setInt8(e, n);
  }
  encodeUint8(e, n) {
    this.data_.setUint8(e, n);
  }
  encodeInt16(e, n) {
    this.data_.setInt16(e, n, mojo.internal.kHostLittleEndian);
  }
  encodeUint16(e, n) {
    this.data_.setUint16(e, n, mojo.internal.kHostLittleEndian);
  }
  encodeInt32(e, n) {
    this.data_.setInt32(e, n, mojo.internal.kHostLittleEndian);
  }
  encodeUint32(e, n) {
    this.data_.setUint32(e, n, mojo.internal.kHostLittleEndian);
  }
  encodeInt64(e, n) {
    mojo.internal.setInt64(this.data_, e, n);
  }
  encodeUint64(e, n) {
    mojo.internal.setUint64(this.data_, e, n);
  }
  encodeFloat(e, n) {
    this.data_.setFloat32(e, n, mojo.internal.kHostLittleEndian);
  }
  encodeDouble(e, n) {
    this.data_.setFloat64(e, n, mojo.internal.kHostLittleEndian);
  }
  encodeHandle(e, n) {
    this.encodeUint32(e, this.message_.handles.length), this.message_.handles.push(n);
  }
  encodeAssociatedEndpoint(e, n) {
    console.assert(n.isPendingAssociation, 'expected unbound associated endpoint');
    const t = this.context_.endpoint.associatePeerOfOutgoingEndpoint(n), o = this.message_.nextInterfaceIdIndex_++;
    this.encodeUint32(e, o), this.message_.interfaceIds_[o] = t;
  }
  encodeString(e, n) {
    if ('string' != typeof n) throw new Error('Unxpected non-string value for string field.');
    this.encodeArray({ elementType: mojo.internal.Uint8 }, e, mojo.internal.Encoder.stringToUtf8Bytes(n));
  }
  encodeOffset(e, n) {
    this.encodeUint64(e, n - this.data_.byteOffset - e);
  }
  encodeArray(e, n, t) {
    const o = mojo.internal.computeInlineArraySize(e, t), i = this.message_.allocate(o), r = new mojo.internal.Encoder(this.message_, i, this.context_);
    this.encodeOffset(n, i.byteOffset), r.encodeUint32(0, o), r.encodeUint32(4, t.length), this.maybeEncodeHasValueBitfield(e, r, 8, t);
    let s = 8 + mojo.internal.computeHasValueBitfieldSize(e, t.length);
    if (e.elementType === mojo.internal.Bool) {
      let e = 0;
      for (const n of t) r.encodeBool(s, e, n), e++, 8 == e && (e = 0, s++);
    } else for (const n of t) {
      if (null === n) {
        if (!e.elementNullable) throw new Error('Trying to send a null element in an array of non-nullable elements');
        e.elementType.$.encodeNull(r, s);
      } else e.elementType.$.encode(n, r, s, 0, !!e.elementNullable);
      s += e.elementType.$.arrayElementSize(!!e.elementNullable);
    }
  }
  maybeEncodeHasValueBitfield(e, n, t, o) {
    if (!e.elementNullable || !e.elementType.$.isValueType) return;
    let i = 0, r = t;
    for (const e of o) null == e ? n.encodeBool(r, i, !1) : n.encodeBool(r, i, !0), i++, 8 == i && (i = 0, r++);
  }
  encodeMap(e, n, t) {
    let o, i;
    'Map' == t.constructor.name ? (o = Array.from(t.keys()), i = Array.from(t.values())) : (o = Object.keys(t), i = o.map(((e) => t[e])));
    const r = this.message_.allocate(mojo.internal.kMapDataSize), s = new mojo.internal.Encoder(this.message_, r, this.context_);
    this.encodeOffset(n, r.byteOffset), s.encodeUint32(0, mojo.internal.kMapDataSize), s.encodeUint32(4, 0), s.encodeArray({ elementType: e.keyType }, 8, o), s.encodeArray({
      elementType: e.valueType,
      elementNullable: e.valueNullable
    }, 16, i);
  }
  encodeStruct(e, n, t) {
    const o = this.message_.allocate(e.packedSize), i = new mojo.internal.Encoder(this.message_, o, this.context_);
    this.encodeOffset(n, o.byteOffset), i.encodeStructInline(e, t);
  }
  encodeStructInline(e, n) {
    const t = e.versions;
    this.encodeUint32(0, e.packedSize), this.encodeUint32(4, t[t.length - 1].version);
    for (const t of e.fields) {
      const o = mojo.internal.kStructHeaderSize + t.packedOffset, i = (e) => {
        t.type.$.encode(e, this, o, t.packedBitOffset, t.nullable);
      }, r = mojo.internal.isNullOrUndefined(n) ? void 0 : mojo.internal.getMojoFieldValue(n, t);
      if (mojo.internal.isNullOrUndefined(n) || mojo.internal.isNullOrUndefined(r)) if (null === t.defaultValue) {
        if (!t.nullable) throw new Error(e.name + ' missing value for non-nullable field "' + t.name + `", got: "${r}"...`);
        t.type.$.encodeNull(this, o);
      } else i(t.defaultValue);
      else i(r);
    }
  }
  encodeUnionAsPointer(e, n, t) {
    const o = this.message_.allocate(mojo.internal.kUnionDataSize), i = new mojo.internal.Encoder(this.message_, o, this.context_);
    this.encodeOffset(n, o.byteOffset), i.encodeUnion(e, 0, t);
  }
  encodeUnion(e, n, t) {
    const o = Object.keys(t);
    if (1 !== o.length) throw new Error(`Value for ${e.name} must be an Object with a single property named one of: ` + Object.keys(e.fields).join(','));
    const i = o[0], r = e.fields[i];
    this.encodeUint32(n, mojo.internal.kUnionDataSize), this.encodeUint32(n + 4, r.ordinal);
    const s = n + mojo.internal.kUnionHeaderSize;
    void 0 === r.type.$.unionSpec ? r.type.$.encode(t[i], this, s, 0, r.nullable) : this.encodeUnionAsPointer(r.type.$.unionSpec, s, t[i]);
  }
  static stringToUtf8Bytes(e) {
    return mojo.internal.Encoder.textEncoder || (mojo.internal.Encoder.textEncoder = new TextEncoder('utf-8')), mojo.internal.Encoder.textEncoder.encode(e);
  }
}, mojo.internal.Encoder.textEncoder = null, mojo.internal.Decoder = class {
  constructor(e, n, t = null) {
    this.context_ = t, this.data_ = e instanceof mojo.internal.DataViewWrapper ? e : new mojo.internal.DataViewWrapper(e), this.handles_ = n;
  }
  decodeBool(e, n) {
    return !!(this.data_.getUint8(e) & 1 << n);
  }
  decodeInt8(e) {
    return this.data_.getInt8(e);
  }
  decodeUint8(e) {
    return this.data_.getUint8(e);
  }
  decodeInt16(e) {
    return this.data_.getInt16(e, mojo.internal.kHostLittleEndian);
  }
  decodeUint16(e) {
    return this.data_.getUint16(e, mojo.internal.kHostLittleEndian);
  }
  decodeInt32(e) {
    return this.data_.getInt32(e, mojo.internal.kHostLittleEndian);
  }
  decodeUint32(e) {
    return this.data_.getUint32(e, mojo.internal.kHostLittleEndian);
  }
  decodeInt64(e) {
    return mojo.internal.getInt64(this.data_, e);
  }
  decodeUint64(e) {
    return mojo.internal.getUint64(this.data_, e);
  }
  decodeFloat(e) {
    return this.data_.getFloat32(e, mojo.internal.kHostLittleEndian);
  }
  decodeDouble(e) {
    return this.data_.getFloat64(e, mojo.internal.kHostLittleEndian);
  }
  decodeHandle(e) {
    const n = this.data_.getUint32(e, mojo.internal.kHostLittleEndian);
    if (4294967295 == n) return null;
    if (n >= this.handles_.length) throw new Error('Decoded invalid handle index');
    return this.handles_[n];
  }
  decodeString(e) {
    const n = this.decodeOffset(e);
    if (!n) return null;
    mojo.internal.Decoder.textDecoder || (mojo.internal.Decoder.textDecoder = new TextDecoder('utf-8'));
    const t = this.decodeUint32(n - this.data_.byteOffset + 4);
    if (0 === t) return '';
    const o = n + 8;
    return mojo.internal.Decoder.textDecoder.decode(new Uint8Array(this.data_.buffer, o, t));
  }
  decodeOffset(e) {
    const n = this.decodeUint64(e);
    if (0 == n) return 0;
    if (n > BigInt(Number.MAX_SAFE_INTEGER)) throw new Error('Mesage offset too large');
    return this.data_.byteOffset + e + Number(n);
  }
  decodeArray(e, n) {
    const t = this.decodeOffset(n);
    if (!t) return null;
    const o = new mojo.internal.Decoder(new mojo.internal.DataViewWrapper(this.data_, t - this.data_.byteOffset), this.handles_, this.context_), i = o.decodeUint32(4);
    if (!i) return [];
    const r = !!e.elementNullable && e.elementType.$.isValueType, s = r ? new Array(i) : null;
    if (r) {
      let e = 8, n = 0;
      for (let t = 0; t < i; ++t) s[t] = o.decodeBool(e, n), n++, 8 === n && (n = 0, e++);
    }
    let a = 8 + mojo.internal.computeHasValueBitfieldSize(e, i);
    const l = new Array(i);
    if (e.elementType === mojo.internal.Bool) for (let e = 0; e < i; ++e) r && !s[e] ? l[e] = null : l[e] = o.decodeBool(a + (e >> 3), e % 8);
    else for (let n = 0; n < i; ++n) {
      if (r && !s[n]) l[n] = null;
      else {
        const t = e.elementType.$.decode(o, a, 0, !!e.elementNullable);
        if (null === t && !e.elementNullable) throw new Error('Received unexpected array element');
        l[n] = t;
      }
      a += e.elementType.$.arrayElementSize(!!e.elementNullable);
    }
    return l;
  }
  decodeMap(e, n) {
    const t = this.decodeOffset(n);
    if (!t) return null;
    const o = new mojo.internal.Decoder(new mojo.internal.DataViewWrapper(this.data_, t - this.data_.byteOffset), this.handles_, this.context_), i = o.decodeUint32(0), r = o.decodeUint32(4);
    if (i != mojo.internal.kMapDataSize || 0 != r) throw new Error('Received invalid map data');
    const s = o.decodeArray({ elementType: e.keyType }, 8), a = o.decodeArray({
      elementType: e.valueType,
      elementNullable: e.valueNullable
    }, 16);
    if (s.length != a.length) throw new Error('Received invalid map data');
    if (!e.keyType.$.isValidObjectKeyType) {
      const e = new Map();
      for (let n = 0; n < s.length; ++n) e.set(s[n], a[n]);
      return e;
    }
    const l = {};
    for (let e = 0; e < s.length; ++e) l[s[e]] = a[e];
    return l;
  }
  decodeStruct(e, n) {
    const t = this.decodeOffset(n);
    if (!t) return null;
    return new mojo.internal.Decoder(new mojo.internal.DataViewWrapper(this.data_, t - this.data_.byteOffset), this.handles_, this.context_).decodeStructInline(e);
  }
  isStructHeaderValid(e, n, t) {
    const o = e.versions;
    for (let e = o.length - 1; e >= 0; --e) {
      const i = o[e];
      if (t > i.version) return n >= i.packedSize;
      if (t == i.version) return n == i.packedSize;
    }
    throw new Error(`Impossible version ${t} for struct ${e.name}`);
  }
  wrapStructInDataView(e, n, t) {
    const o = this.decodeOffset(n);
    if (!o) return null;
    const i = new mojo.internal.Decoder(new mojo.internal.DataViewWrapper(this.data_, o - this.data_.byteOffset), this.handles_, this.context_), r = i.decodeUint32(mojo.internal.kStructHeaderSizeOffset), s = i.decodeUint32(mojo.internal.kStructHeaderVersionOffset);
    if (!i.isStructHeaderValid(e, r, s)) throw new Error(`Received ${e.name} of invalid size (${r}) and/or version (${s})`);
    return new t(i, s, e.fields);
  }
  decodeStructInline(e) {
    const n = this.decodeUint32(mojo.internal.kStructHeaderSizeOffset), t = this.decodeUint32(mojo.internal.kStructHeaderVersionOffset);
    if (!this.isStructHeaderValid(e, n, t)) throw new Error(`Received ${e.name} of invalid size (${n}) and/or version (${t})`);
    const o = {};
    for (let n = 0; n < e.fields.length; ++n) {
      const i = e.fields[n];
      if (mojo.internal.isNullableValueKindField(i)) {
        const n = i.nullableValueKindProperties;
        if (!n.isPrimary) continue;
        {
          const r = i;
          o[n.originalFieldName] = this.decodeStructNullableValueField(r, e.fields, t);
        }
      } else o[i.name] = this.decodeStructField(i, t);
    }
    return o;
  }
  decodeStructField(e, n) {
    if (e.minVersion > n) return e.defaultValue;
    const t = mojo.internal.kStructHeaderSize + e.packedOffset, o = e.type.$.decode(this, t, e.packedBitOffset, !!e.nullable);
    if (null === o && !e.nullable) throw new Error(`Received ${e.name} with invalid null field "${e.name}"`);
    return o;
  }
  decodeStructNullableValueField(e, n, t) {
    if (e.minVersion > t) return null;
    if (!this.decodeStructField(e, t)) return null;
    const o = e.nullableValueKindProperties, i = n.find(((e) => e.name === o.linkedValueFieldName));
    if (!i) throw new Error('could not find the expected value field spec: ' + o.linkedValueFieldName);
    return this.decodeStructField(i, t);
  }
  decodeUnionFromPointer(e, n) {
    const t = this.decodeOffset(n);
    if (!t) return null;
    return new mojo.internal.Decoder(new mojo.internal.DataViewWrapper(this.data_, t - this.data_.byteOffset), this.handles_, this.context_).decodeUnion(e, 0);
  }
  decodeUnion(e, n) {
    if (0 === this.decodeUint32(n)) return null;
    const t = this.decodeUint32(n + 4);
    for (const o in e.fields) {
      const i = e.fields[o];
      if (i.ordinal === t) {
        const t = (() => {
          const e = n + mojo.internal.kUnionHeaderSize;
          return void 0 !== i.type.$.unionSpec ? this.decodeUnionFromPointer(i.type.$.unionSpec, e) : i.type.$.decode(this, e, 0, i.nullable);
        })();
        if (null === t && !i.nullable) throw new Error(`Received ${e.name} with invalid null field: ${i.name}`);
        const r = {};
        return r[o] = t, r;
      }
    }
  }
  decodeInterfaceProxy(e, n) {
    const t = this.decodeHandle(n);
    return t ? new e(t) : null;
  }
  decodeInterfaceRequest(e, n) {
    const t = this.decodeHandle(n);
    return t ? new e(mojo.internal.interfaceSupport.createEndpoint(t)) : null;
  }
  decodeAssociatedEndpoint(e) {
    if (!this.context_ || !this.context_.endpoint) throw new Error('cannot deserialize associated endpoint without context');
    const n = this.context_.endpoint, t = new DataView(this.data_.buffer), o = Number(mojo.internal.getUint64(t, 40)), i = t.getUint32(o + 44, mojo.internal.kHostLittleEndian), r = new Uint32Array(t.buffer, o + mojo.internal.kArrayHeaderSize + 40, i)[this.decodeUint32(e)], s = new mojo.internal.interfaceSupport.Endpoint(n.router, r);
    return n.router.addEndpoint(s, r), s;
  }
}, mojo.internal.Decoder.textDecoder = null, mojo.internal.MessageHeader, mojo.internal.deserializeMessageHeader = function(e) {
  const n = e.getUint32(0, mojo.internal.kHostLittleEndian), t = e.getUint32(4, mojo.internal.kHostLittleEndian);
  if (0 == t && n != mojo.internal.kMessageV0HeaderSize || 1 == t && n != mojo.internal.kMessageV1HeaderSize || t >= 2 && n < mojo.internal.kMessageV2HeaderSize) throw new Error('Received invalid message header');
  return {
    headerSize: n,
    headerVersion: t,
    interfaceId: e.getUint32(8, mojo.internal.kHostLittleEndian),
    ordinal: e.getUint32(12, mojo.internal.kHostLittleEndian),
    flags: e.getUint32(16, mojo.internal.kHostLittleEndian),
    requestId: t < 1 ? 0 : e.getUint32(24, mojo.internal.kHostLittleEndian)
  };
}, mojo.internal.MojomTypeInfo, mojo.internal.MojomType, mojo.internal.ArraySpec, mojo.internal.MapSpec, mojo.internal.NullableValueKindProperties = class {
  constructor() {
    this.isPrimary, this.linkedValueFieldName, this.originalFieldName;
  }
}, mojo.internal.StructFieldSpec, mojo.internal.StructVersionInfo, mojo.internal.StructSpec, mojo.internal.UnionFieldSpec, mojo.internal.UnionSpec, mojo.internal.Bool = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeBool(t, o, e);
  },
  encodeNull: function(e, n) {
    throw new Error('encoding bool null from type is not implemented');
  },
  decode: function(e, n, t, o) {
    return e.decodeBool(n, t);
  },
  arrayElementSize: (e) => 1,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Int8 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeInt8(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeInt8(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeInt8(n);
  },
  arrayElementSize: (e) => 1,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Uint8 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeUint8(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeUint8(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeUint8(n);
  },
  arrayElementSize: (e) => 1,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Int16 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeInt16(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeInt16(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeInt16(n);
  },
  arrayElementSize: (e) => 2,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Uint16 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeUint16(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeUint16(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeUint16(n);
  },
  arrayElementSize: (e) => 2,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Int32 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeInt32(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeInt32(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeInt32(n);
  },
  arrayElementSize: (e) => 4,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Uint32 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeUint32(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeUint32(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeUint32(n);
  },
  arrayElementSize: (e) => 4,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Int64 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeInt64(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeInt64(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeInt64(n);
  },
  arrayElementSize: (e) => 8,
  isValidObjectKeyType: !1,
  isValueType: !0
} }, mojo.internal.Uint64 = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeUint64(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeUint64(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeUint64(n);
  },
  arrayElementSize: (e) => 8,
  isValidObjectKeyType: !1,
  isValueType: !0
} }, mojo.internal.Float = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeFloat(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeFloat(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeFloat(n);
  },
  arrayElementSize: (e) => 4,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Double = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeDouble(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeDouble(n, 0);
  },
  decode: function(e, n, t, o) {
    return e.decodeDouble(n);
  },
  arrayElementSize: (e) => 8,
  isValidObjectKeyType: !0,
  isValueType: !0
} }, mojo.internal.Handle = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeHandle(t, e);
  },
  encodeNull: function(e, n) {
    e.encodeUint32(n, 4294967295);
  },
  decode: function(e, n, t, o) {
    return e.decodeHandle(n);
  },
  arrayElementSize: (e) => 4,
  isValidObjectKeyType: !1,
  isValueType: !1
} }, mojo.internal.String = { $: {
  encode: function(e, n, t, o, i) {
    n.encodeString(t, e);
  },
  encodeNull: function(e, n) {},
  decode: function(e, n, t, o) {
    return e.decodeString(n);
  },
  computeDimensions: function(e, n) {
    return { size: mojo.internal.computeTotalArraySize({ elementType: mojo.internal.Uint8 }, mojo.internal.Encoder.stringToUtf8Bytes(e)) };
  },
  arrayElementSize: (e) => 8,
  isValidObjectKeyType: !0,
  isValueType: !1
} }, mojo.internal.Array = function(e, n) {
  const t = {
    elementType: e,
    elementNullable: n
  };
  return { $: {
    arraySpec: t,
    encode: function(e, n, o, i, r) {
      n.encodeArray(t, o, e);
    },
    encodeNull: function(e, n) {},
    decode: function(e, n, o, i) {
      return e.decodeArray(t, n);
    },
    computeDimensions: function(e, n) {
      return { size: mojo.internal.computeTotalArraySize(t, e) };
    },
    arrayElementSize: (e) => 8,
    isValidObjectKeyType: !1,
    isValueType: !1
  } };
}, mojo.internal.Map = function(e, n, t) {
  const o = {
    keyType: e,
    valueType: n,
    valueNullable: t
  };
  return { $: {
    mapSpec: o,
    encode: function(e, n, t, i, r) {
      n.encodeMap(o, t, e);
    },
    encodeNull: function(e, n) {},
    decode: function(e, n, t, i) {
      return e.decodeMap(o, n);
    },
    computeDimensions: function(o, i) {
      const r = 'Map' == o.constructor.name ? Array.from(o.keys()) : Object.keys(o), s = 'Map' == o.constructor.name ? Array.from(o.values()) : r.map(((e) => o[e]));
      return { size: mojo.internal.kMapDataSize + mojo.internal.align(mojo.internal.computeTotalArraySize({ elementType: e }, r), 8) + mojo.internal.computeTotalArraySize({
        elementType: n,
        elementNullable: t
      }, s) };
    },
    arrayElementSize: (e) => 8,
    isValidObjectKeyType: !1,
    isValueType: !1
  } };
}, mojo.internal.Enum = function() {
  return { $: {
    encode: function(e, n, t, o, i) {
      n.encodeUint32(t, e);
    },
    encodeNull: function(e, n) {},
    decode: function(e, n, t, o) {
      return e.decodeInt32(n);
    },
    arrayElementSize: (e) => 4,
    isValidObjectKeyType: !0,
    isValueType: !0
  } };
}, mojo.internal.StructField = function(e, n, t, o, i, r, s = 0, a = void 0, l = void 0) {
  return {
    name: e,
    packedOffset: n,
    packedBitOffset: t,
    type: o,
    defaultValue: i,
    nullable: r,
    minVersion: s,
    nullableValueKindProperties: a,
    fieldGetter: l
  };
}, mojo.internal.Struct = function(e, n, t, o) {
  const i = o.map(((e) => ({
    version: e[0],
    packedSize: e[1]
  }))), r = {
    name: n,
    packedSize: i[i.length - 1].packedSize,
    fields: t,
    versions: i
  };
  e.$ = {
    structSpec: r,
    encode: function(e, n, t, o, i) {
      n.encodeStruct(r, t, e);
    },
    encodeNull: function(e, n) {},
    decode: function(e, n, t, o) {
      return e.decodeStruct(r, n);
    },
    computeDimensions: function(e, n) {
      return mojo.internal.computeStructDimensions(r, e);
    },
    arrayElementSize: (e) => 8,
    isValidObjectKeyType: !1
  };
}, mojo.internal.TypemappedStruct = function(e, n, t, o, i, r) {
  const s = r.map(((e) => ({
    version: e[0],
    packedSize: e[1]
  }))), a = {
    name: n,
    packedSize: s[s.length - 1].packedSize,
    fields: i,
    versions: s
  };
  e.$ = {
    structSpec: a,
    encode: function(e, n, t, o, i) {
      n.encodeStruct(a, t, e);
    },
    encodeNull: function(e, n) {},
    decode: function(e, n, i, r) {
      const s = e.wrapStructInDataView(a, n, t);
      return mojo.internal.isNullOrUndefined(s) ? null : o.convert(s);
    },
    computeDimensions: function(e, n) {
      return mojo.internal.computeStructDimensions(a, e);
    },
    arrayElementSize: (e) => 8,
    isValidObjectKeyType: !1
  };
}, mojo.internal.createStructDeserializer = function(e) {
  return function(n) {
    if (null == e.$ || null == e.$.structSpec) throw new Error('Invalid struct mojom type!');
    return new mojo.internal.Decoder(n, []).decodeStructInline(e.$.structSpec);
  };
}, mojo.internal.Union = function(e, n, t) {
  const o = {
    name: n,
    fields: t
  };
  e.$ = {
    unionSpec: o,
    encode: function(e, n, t, i, r) {
      n.encodeUnion(o, t, e);
    },
    encodeNull: function(e, n) {},
    decode: function(e, n, t, i) {
      return e.decodeUnion(o, n);
    },
    computeDimensions: function(e, n) {
      return mojo.internal.computeUnionDimensions(o, n, e);
    },
    arrayElementSize: (e) => e ? 8 : 16,
    isValidObjectKeyType: !1
  };
}, mojo.internal.InterfaceProxy = function(e) {
  return { $: {
    encode: function(n, t, o, i, r) {
      const s = n.proxy.unbind();
      console.assert(s, `unexpected null ${e.name}`);
      const a = s.releasePipe();
      t.encodeHandle(o, a), t.encodeUint32(o + 4, 0);
    },
    encodeNull: function(e, n) {
      e.encodeUint32(n, 4294967295);
    },
    decode: function(n, t, o, i) {
      return n.decodeInterfaceProxy(e, t);
    },
    arrayElementSize: (e) => 8,
    isValidObjectKeyType: !1,
    isValueType: !1
  } };
}, mojo.internal.InterfaceRequest = function(e) {
  return { $: {
    encode: function(n, t, o, i, r) {
      if (!n.handle) throw new Error('Unexpected null ' + e.name);
      t.encodeHandle(o, n.handle.releasePipe());
    },
    encodeNull: function(e, n) {
      e.encodeUint32(n, 4294967295);
    },
    decode: function(n, t, o, i) {
      return n.decodeInterfaceRequest(e, t);
    },
    arrayElementSize: (e) => 8,
    isValidObjectKeyType: !1,
    isValueType: !1
  } };
}, mojo.internal.AssociatedInterfaceProxy = function(e) {
  return { $: {
    type: e,
    encode: function(n, t, o, i, r) {
      console.assert(n.proxy.endpoint && n.proxy.endpoint.isPendingAssociation, `expected ${e.name} to be associated and unbound`), t.encodeAssociatedEndpoint(o, n.proxy.endpoint), t.encodeUint32(o + 4, 0);
    },
    encodeNull: function(e, n) {
      e.encodeUint32(n, 4294967295), e.encodeUint32(n + 4, 0);
    },
    decode: function(n, t, o, i) {
      return new e(n.decodeAssociatedEndpoint(t));
    },
    arrayElementSize: (e) => {
      throw new Error('Arrays of associated endpoints are not yet supported');
    },
    isValidObjectKeyType: !1,
    hasInterfaceId: !0,
    isValueType: !1
  } };
}, mojo.internal.AssociatedInterfaceRequest = function(e) {
  return { $: {
    type: e,
    encode: function(n, t, o, i, r) {
      console.assert(n.handle && n.handle.isPendingAssociation, `expected ${e.name} to be associated and unbound`), t.encodeAssociatedEndpoint(o, n.handle);
    },
    encodeNull: function(e, n) {
      e.encodeUint32(n, 4294967295);
    },
    decode: function(n, t, o, i) {
      return new e(n.decodeAssociatedEndpoint(t));
    },
    arrayElementSize: (e) => {
      throw new Error('Arrays of associated endpoints are not yet supported');
    },
    isValidObjectKeyType: !1,
    hasInterfaceId: !0,
    isValueType: !1
  } };
}, mojo.internal.decodeStructField = function(e, n, t) {
  return e.decodeStructField(n, t);
}, mojo.internal.decodeStructNullableValueField = function(e, n, t, o) {
  n.nullableValueKindProperties.linkedValueFieldName;
  return e.decodeStructNullableValueField(n, t, o);
}, mojo.interfaceControl.RUN_MESSAGE_ID = 4294967295, mojo.interfaceControl.RUN_OR_CLOSE_PIPE_MESSAGE_ID = 4294967294, mojo.interfaceControl.RunMessageParamsSpec = { $: {} }, mojo.interfaceControl.RunResponseMessageParamsSpec = { $: {} }, mojo.interfaceControl.QueryVersionSpec = { $: {} }, mojo.interfaceControl.QueryVersionResultSpec = { $: {} }, mojo.interfaceControl.FlushForTestingSpec = { $: {} }, mojo.interfaceControl.RunOrClosePipeMessageParamsSpec = { $: {} }, mojo.interfaceControl.RequireVersionSpec = { $: {} }, mojo.interfaceControl.EnableIdleTrackingSpec = { $: {} }, mojo.interfaceControl.MessageAckSpec = { $: {} }, mojo.interfaceControl.NotifyIdleSpec = { $: {} }, mojo.interfaceControl.RunInputSpec = { $: {} }, mojo.interfaceControl.RunOutputSpec = { $: {} }, mojo.interfaceControl.RunOrClosePipeInputSpec = { $: {} }, mojo.internal.Struct(mojo.interfaceControl.RunMessageParamsSpec.$, 'RunMessageParams', [mojo.internal.StructField('input', 0, 0, mojo.interfaceControl.RunInputSpec.$, null, !1, 0)], [[0, 24]]), mojo.interfaceControl.RunMessageParams = class {
  constructor() {
    this.input;
  }
}, mojo.internal.Struct(mojo.interfaceControl.RunResponseMessageParamsSpec.$, 'RunResponseMessageParams', [mojo.internal.StructField('output', 0, 0, mojo.interfaceControl.RunOutputSpec.$, null, !0, 0)], [[0, 24]]), mojo.interfaceControl.RunResponseMessageParams = class {
  constructor() {
    this.output;
  }
}, mojo.internal.Struct(mojo.interfaceControl.QueryVersionSpec.$, 'QueryVersion', [], [[0, 8]]), mojo.interfaceControl.QueryVersion = class {
  constructor() {}
}, mojo.internal.Struct(mojo.interfaceControl.QueryVersionResultSpec.$, 'QueryVersionResult', [mojo.internal.StructField('version', 0, 0, mojo.internal.Uint32, 0, !1, 0)], [[0, 16]]), mojo.interfaceControl.QueryVersionResult = class {
  constructor() {
    this.version;
  }
}, mojo.internal.Struct(mojo.interfaceControl.FlushForTestingSpec.$, 'FlushForTesting', [], [[0, 8]]), mojo.interfaceControl.FlushForTesting = class {
  constructor() {}
}, mojo.internal.Struct(mojo.interfaceControl.RunOrClosePipeMessageParamsSpec.$, 'RunOrClosePipeMessageParams', [mojo.internal.StructField('input', 0, 0, mojo.interfaceControl.RunOrClosePipeInputSpec.$, null, !1, 0)], [[0, 24]]), mojo.interfaceControl.RunOrClosePipeMessageParams = class {
  constructor() {
    this.input;
  }
}, mojo.internal.Struct(mojo.interfaceControl.RequireVersionSpec.$, 'RequireVersion', [mojo.internal.StructField('version', 0, 0, mojo.internal.Uint32, 0, !1, 0)], [[0, 16]]), mojo.interfaceControl.RequireVersion = class {
  constructor() {
    this.version;
  }
}, mojo.internal.Struct(mojo.interfaceControl.EnableIdleTrackingSpec.$, 'EnableIdleTracking', [mojo.internal.StructField('timeoutInMicroseconds', 0, 0, mojo.internal.Int64, BigInt(0), !1, 0)], [[0, 16]]), mojo.interfaceControl.EnableIdleTracking = class {
  constructor() {
    this.timeoutInMicroseconds;
  }
}, mojo.internal.Struct(mojo.interfaceControl.MessageAckSpec.$, 'MessageAck', [], [[0, 8]]), mojo.interfaceControl.MessageAck = class {
  constructor() {}
}, mojo.internal.Struct(mojo.interfaceControl.NotifyIdleSpec.$, 'NotifyIdle', [], [[0, 8]]), mojo.interfaceControl.NotifyIdle = class {
  constructor() {}
}, mojo.internal.Union(mojo.interfaceControl.RunInputSpec.$, 'RunInput', {
  queryVersion: {
    ordinal: 0,
    type: mojo.interfaceControl.QueryVersionSpec.$
  },
  flushForTesting: {
    ordinal: 1,
    type: mojo.interfaceControl.FlushForTestingSpec.$
  }
}), mojo.interfaceControl.RunInput, mojo.internal.Union(mojo.interfaceControl.RunOutputSpec.$, 'RunOutput', { queryVersionResult: {
  ordinal: 0,
  type: mojo.interfaceControl.QueryVersionResultSpec.$
} }), mojo.interfaceControl.RunOutput, mojo.internal.Union(mojo.interfaceControl.RunOrClosePipeInputSpec.$, 'RunOrClosePipeInput', {
  requireVersion: {
    ordinal: 0,
    type: mojo.interfaceControl.RequireVersionSpec.$
  },
  enableIdleTracking: {
    ordinal: 1,
    type: mojo.interfaceControl.EnableIdleTrackingSpec.$
  },
  messageAck: {
    ordinal: 2,
    type: mojo.interfaceControl.MessageAckSpec.$
  },
  notifyIdle: {
    ordinal: 3,
    type: mojo.interfaceControl.NotifyIdleSpec.$
  }
}), mojo.interfaceControl.RunOrClosePipeInput, mojo.pipeControl.RUN_OR_CLOSE_PIPE_MESSAGE_ID = 4294967294, mojo.pipeControl.RunOrClosePipeMessageParamsSpec = { $: {} }, mojo.pipeControl.DisconnectReasonSpec = { $: {} }, mojo.pipeControl.PeerAssociatedEndpointClosedEventSpec = { $: {} }, mojo.pipeControl.PauseUntilFlushCompletesSpec = { $: {} }, mojo.pipeControl.FlushAsyncSpec = { $: {} }, mojo.pipeControl.RunOrClosePipeInputSpec = { $: {} }, mojo.internal.Struct(mojo.pipeControl.RunOrClosePipeMessageParamsSpec.$, 'RunOrClosePipeMessageParams', [mojo.internal.StructField('input', 0, 0, mojo.pipeControl.RunOrClosePipeInputSpec.$, null, !1, 0)], [[0, 24]]), mojo.pipeControl.RunOrClosePipeMessageParams = class {
  constructor() {
    this.input;
  }
}, mojo.internal.Struct(mojo.pipeControl.DisconnectReasonSpec.$, 'DisconnectReason', [mojo.internal.StructField('customReason', 0, 0, mojo.internal.Uint32, 0, !1, 0), mojo.internal.StructField('description', 8, 0, mojo.internal.String, null, !1, 0)], [[0, 24]]), mojo.pipeControl.DisconnectReason = class {
  constructor() {
    this.customReason, this.description;
  }
}, mojo.internal.Struct(mojo.pipeControl.PeerAssociatedEndpointClosedEventSpec.$, 'PeerAssociatedEndpointClosedEvent', [mojo.internal.StructField('id', 0, 0, mojo.internal.Uint32, 0, !1, 0), mojo.internal.StructField('disconnectReason', 8, 0, mojo.pipeControl.DisconnectReasonSpec.$, null, !0, 0)], [[0, 24]]), mojo.pipeControl.PeerAssociatedEndpointClosedEvent = class {
  constructor() {
    this.id, this.disconnectReason;
  }
}, mojo.internal.Struct(mojo.pipeControl.PauseUntilFlushCompletesSpec.$, 'PauseUntilFlushCompletes', [mojo.internal.StructField('flushPipe', 0, 0, mojo.internal.Handle, null, !1, 0)], [[0, 16]]), mojo.pipeControl.PauseUntilFlushCompletes = class {
  constructor() {
    this.flushPipe;
  }
}, mojo.internal.Struct(mojo.pipeControl.FlushAsyncSpec.$, 'FlushAsync', [mojo.internal.StructField('flusherPipe', 0, 0, mojo.internal.Handle, null, !1, 0)], [[0, 16]]), mojo.pipeControl.FlushAsync = class {
  constructor() {
    this.flusherPipe;
  }
}, mojo.internal.Union(mojo.pipeControl.RunOrClosePipeInputSpec.$, 'RunOrClosePipeInput', {
  peerAssociatedEndpointClosedEvent: {
    ordinal: 0,
    type: mojo.pipeControl.PeerAssociatedEndpointClosedEventSpec.$
  },
  pauseUntilFlushCompletes: {
    ordinal: 1,
    type: mojo.pipeControl.PauseUntilFlushCompletesSpec.$
  },
  flushAsync: {
    ordinal: 2,
    type: mojo.pipeControl.FlushAsyncSpec.$
  }
}), mojo.pipeControl.RunOrClosePipeInput, mojo.internal.interfaceSupport.RouterMessage, mojo.internal.interfaceSupport.Router = class {
  constructor(e, n) {
    this.pipe_ = e, this.messages_ = [], this.dispatchInProgress_ = !1, this.reader_ = new mojo.internal.interfaceSupport.HandleReader(e), this.reader_.onRead = this.onMessageReceived_.bind(this), this.reader_.onError = this.onError_.bind(this), this.endpoints_ = new Map(), this.nextInterfaceId_ = 1, this.interfaceIdNamespace_ = n ? mojo.internal.kInterfaceNamespaceBit : 0, this.pipeControlHandler_ = new mojo.internal.interfaceSupport.PipeControlMessageHandler(this, this.onPeerEndpointClosed_.bind(this));
  }
  get pipe() {
    return this.pipe_;
  }
  generateInterfaceId() {
    return (this.nextInterfaceId_++ | this.interfaceIdNamespace_) >>> 0;
  }
  addEndpoint(e, n) {
    0 === n && this.reader_.start(), console.assert(this.isReading(), 'adding a secondary endpoint with no primary'), this.endpoints_.set(n, e), this.dispatchMessages_();
  }
  removeEndpoint(e) {
    this.endpoints_.delete(e), 0 === e && this.reader_.stop();
  }
  close() {
    console.assert(0 === this.endpoints_.size, 'closing primary endpoint with secondary endpoints still bound'), this.reader_.stopAndCloseHandle();
  }
  closeEndpoint(e) {
    this.removeEndpoint(e), 0 === e ? this.close() : this.pipeControlHandler_.notifyEndpointClosed(e);
  }
  isReading() {
    return !this.reader_.isStopped();
  }
  send(e) {
    this.pipe_.writeMessage(e.buffer, e.handles);
  }
  onMessageReceived_(e, n) {
    if (!this.checkSize_(e)) return;
    const t = mojo.internal.deserializeMessageHeader(new mojo.internal.DataViewWrapper(e));
    this.pipeControlHandler_.maybeHandleMessage(t, e) || (this.messages_.push({
      header: t,
      buffer: e,
      handles: n
    }), this.dispatchMessages_());
  }
  checkSize_(e) {
    return !(e.byteLength < mojo.internal.kMessageV0HeaderSize) || (console.error('Rejecting undersized message'), this.onError_(), !1);
  }
  dispatchMessages_() {
    if (!this.dispatchInProgress_) {
      for (this.dispatchInProgress_ = !0; this.messages_.length > 0;) {
        const e = this.messages_[0];
        if (!this.dispatch_(e)) break;
        this.messages_.shift();
      }
      this.dispatchInProgress_ = !1;
    }
  }
  dispatch_(e) {
    const n = this.endpoints_.get(e.header.interfaceId);
    return n ? !!n.isStarted && (n.onMessageReceived(e.header, e.buffer, e.handles), !0) : (console.error(`Received message for unknown endpoint ${e.header.interfaceId}`), !1);
  }
  onError_() {
    for (const e of this.endpoints_.values()) e.onError();
    this.endpoints_.clear();
  }
  onPeerEndpointClosed_(e) {
    const n = this.endpoints_.get(e);
    n && n.onError();
  }
}, mojo.internal.interfaceSupport.EndpointClient = class {
  onMessageReceived(e, n, t, o) {}
  onError(e, n = void 0) {}
}, mojo.internal.interfaceSupport.Endpoint = class {
  constructor(e = null, n = 0) {
    this.router_ = e, this.interfaceId_ = n, this.controlMessageHandler_ = new mojo.internal.interfaceSupport.ControlMessageHandler(this), this.client_ = null, this.nextRequestId_ = 0, this.localPeer_ = null;
  }
  static createAssociatedPair() {
    const e = new mojo.internal.interfaceSupport.Endpoint(), n = new mojo.internal.interfaceSupport.Endpoint();
    return n.localPeer_ = e, e.localPeer_ = n, {
      endpoint0: e,
      endpoint1: n
    };
  }
  get router() {
    return this.router_;
  }
  isPrimary() {
    return null !== this.router_ && 0 === this.interfaceId_;
  }
  releasePipe() {
    return console.assert(this.isPrimary(), 'secondary endpoint cannot release pipe'), this.router_.pipe;
  }
  get isPendingAssociation() {
    return null !== this.localPeer_;
  }
  bindInBrowser(e, n) {
    console.assert(this.isPrimary() && !this.router_.isReading(), 'endpoint is either associated or already bound'), Mojo.bindInterface(e, this.router_.pipe, n);
  }
  associatePeerOfOutgoingEndpoint(e) {
    console.assert(this.router_, 'cannot associate with unbound endpoint');
    const n = e.localPeer_;
    e.localPeer_ = n.localPeer_ = null;
    const t = this.router_.generateInterfaceId();
    return n.router_ = this.router_, n.interfaceId_ = t, n.client_ && this.router_.addEndpoint(n, t), t;
  }
  generateRequestId() {
    const e = this.nextRequestId_++;
    return this.nextRequestId_ > 4294967295 && (this.nextRequestId_ = 0), e;
  }
  send(e, n, t, o, i) {
    const r = new mojo.internal.Message(this, this.interfaceId_, t, e, n, o.$.structSpec, i);
    console.assert(this.router_, 'cannot send message on unassociated unbound endpoint'), this.router_.send(r);
  }
  start(e) {
    console.assert(!this.client_, 'endpoint already started'), this.client_ = e, this.router_ && this.router_.addEndpoint(this, this.interfaceId_);
  }
  get isStarted() {
    return null !== this.client_;
  }
  stop() {
    this.router_ && this.router_.removeEndpoint(this.interfaceId_), this.client_ = null, this.controlMessageHandler_ = null;
  }
  close() {
    this.router_ && this.router.closeEndpoint(this.interfaceId_), this.client_ = null, this.controlMessageHandler_ = null;
  }
  async flushForTesting() {
    return this.controlMessageHandler_.sendRunMessage({ flushForTesting: {} });
  }
  onMessageReceived(e, n, t) {
    console.assert(this.client_, 'endpoint has no client');
    this.controlMessageHandler_.maybeHandleControlMessage(e, n) || this.client_.onMessageReceived(this, e, n, t);
  }
  onError() {
    this.client_ && this.client_.onError(this);
  }
}, mojo.internal.interfaceSupport.acceptBufferForTesting = function(e, n) {
  e.router_.onMessageReceived_(n, []);
}, mojo.internal.interfaceSupport.createEndpoint = function(e, n = !1) {
  return void 0 === e.watch ? e : new mojo.internal.interfaceSupport.Endpoint(new mojo.internal.interfaceSupport.Router(e, n), 0);
}, mojo.internal.interfaceSupport.getEndpointForReceiver = function(e) {
  return mojo.internal.interfaceSupport.createEndpoint(e);
}, mojo.internal.interfaceSupport.bind = function(e, n, t) {
  e.bindInBrowser(n, t);
}, mojo.internal.interfaceSupport.PipeControlMessageHandler = class {
  constructor(e, n) {
    this.router_ = e, this.onDisconnect_ = n;
  }
  send(e) {
    const n = new mojo.internal.Message(null, 4294967295, 0, mojo.pipeControl.RUN_OR_CLOSE_PIPE_MESSAGE_ID, 0, mojo.pipeControl.RunOrClosePipeMessageParamsSpec.$.$.structSpec, { input: e });
    this.router_.send(n);
  }
  maybeHandleMessage(e, n) {
    if (e.ordinal !== mojo.pipeControl.RUN_OR_CLOSE_PIPE_MESSAGE_ID) return !1;
    const t = new mojo.internal.DataViewWrapper(n, e.headerSize), o = new mojo.internal.Decoder(t, []), i = mojo.pipeControl.RunOrClosePipeMessageParamsSpec.$.$.structSpec, r = o.decodeStructInline(i).input;
    return !r.hasOwnProperty('peerAssociatedEndpointClosedEvent') || (this.onDisconnect_(r.peerAssociatedEndpointClosedEvent.id), !0);
  }
  notifyEndpointClosed(e) {
    this.send({ peerAssociatedEndpointClosedEvent: { id: e } });
  }
}, mojo.internal.interfaceSupport.ControlMessageHandler = class {
  constructor(e) {
    this.endpoint_ = e, this.pendingFlushResolvers_ = new Map();
  }
  sendRunMessage(e) {
    const n = this.endpoint_.generateRequestId();
    return new Promise(((t) => {
      this.endpoint_.send(mojo.interfaceControl.RUN_MESSAGE_ID, n, mojo.internal.kMessageFlagExpectsResponse, mojo.interfaceControl.RunMessageParamsSpec.$, { input: e }), this.pendingFlushResolvers_.set(n, t);
    }));
  }
  maybeHandleControlMessage(e, n) {
    if (e.ordinal === mojo.interfaceControl.RUN_MESSAGE_ID) {
      const t = new mojo.internal.DataViewWrapper(n, e.headerSize), o = new mojo.internal.Decoder(t, []);
      return e.flags & mojo.internal.kMessageFlagExpectsResponse ? this.handleRunRequest_(e.requestId, o) : this.handleRunResponse_(e.requestId, o);
    }
    return !1;
  }
  handleRunRequest_(e, n) {
    return !!n.decodeStructInline(mojo.interfaceControl.RunMessageParamsSpec.$.$.structSpec).input.hasOwnProperty('flushForTesting') && (this.endpoint_.send(mojo.interfaceControl.RUN_MESSAGE_ID, e, mojo.internal.kMessageFlagIsResponse, mojo.interfaceControl.RunResponseMessageParamsSpec.$, { output: null }), !0);
  }
  handleRunResponse_(e, n) {
    const t = this.pendingFlushResolvers_.get(e);
    return !!t && (t(), !0);
  }
}, mojo.internal.interfaceSupport.PendingResponse, mojo.internal.interfaceSupport.ConnectionErrorEventRouter = class {
  constructor() {
    this.listeners = new Map(), this.nextListenerId_ = 0;
  }
  addListener(e) {
    const n = ++this.nextListenerId_;
    return this.listeners.set(n, e), n;
  }
  removeListener(e) {
    return this.listeners.delete(e);
  }
  dispatchErrorEvent() {
    for (const e of this.listeners.values()) e();
  }
}, mojo.internal.interfaceSupport.PendingReceiver = class {
  get handle() {}
}, mojo.internal.interfaceSupport.InterfaceRemoteBase = class {
  constructor(e, n = void 0) {
    this.endpoint_ = null, this.requestType_ = e, this.pendingResponses_ = new Map(), this.connectionErrorEventRouter_ = new mojo.internal.interfaceSupport.ConnectionErrorEventRouter(), n && this.bindHandle(n);
  }
  get endpoint() {
    return this.endpoint_;
  }
  bindNewPipeAndPassReceiver() {
    let { handle0: e, handle1: n } = Mojo.createMessagePipe();
    return this.bindHandle(e), new this.requestType_(mojo.internal.interfaceSupport.createEndpoint(n));
  }
  bindHandle(e) {
    console.assert(!this.endpoint_, 'already bound'), e = mojo.internal.interfaceSupport.createEndpoint(e, !0), this.endpoint_ = e, this.endpoint_.start(this), this.pendingResponses_ = new Map();
  }
  associateAndPassReceiver() {
    console.assert(!this.endpoint_, 'cannot associate when already bound');
    const { endpoint0: e, endpoint1: n } = mojo.internal.interfaceSupport.Endpoint.createAssociatedPair();
    return this.bindHandle(e), new this.requestType_(n);
  }
  unbind() {
    if (!this.endpoint_) return null;
    const e = this.endpoint_;
    return this.endpoint_ = null, e.stop(), e;
  }
  close() {
    this.cleanupAndFlushPendingResponses_('Message pipe closed.'), this.endpoint_ && this.endpoint_.close(), this.endpoint_ = null;
  }
  getConnectionErrorEventRouter() {
    return this.connectionErrorEventRouter_;
  }
  sendMessage(e, n, t, o, i) {
    if (t && (!this.endpoint_ || !this.endpoint_.isStarted)) return Promise.reject(new Error('The pipe has already been closed.'));
    const r = {};
    let s = 0;
    n.$.structSpec.fields.forEach(((e, n) => {
      const t = n - s;
      if (!mojo.internal.isNullableValueKindField(e)) return void (r[e.name] = o[t]);
      const i = e.nullableValueKindProperties;
      i.isPrimary && (s++, r[i.originalFieldName] = o[t]);
    }));
    const a = this.endpoint_.generateRequestId();
    if (this.endpoint_.send(e, a, t ? mojo.internal.kMessageFlagExpectsResponse : 0, n, r), !t) return Promise.resolve();
    const l = t;
    return new Promise(((n, t) => {
      this.pendingResponses_.set(a, {
        requestId: a,
        ordinal: e,
        responseStruct: l,
        resolve: n,
        reject: t,
        useResultResponse: i
      });
    }));
  }
  flushForTesting() {
    return this.endpoint_.flushForTesting();
  }
  onMessageReceived(e, n, t, o) {
    if (!(n.flags & mojo.internal.kMessageFlagIsResponse) || n.flags & mojo.internal.kMessageFlagExpectsResponse) return this.onError(e, 'Received unexpected request message');
    const i = this.pendingResponses_.get(n.requestId);
    if (this.pendingResponses_.delete(n.requestId), !i) return this.onError(e, 'Received unexpected response message');
    const r = new mojo.internal.Decoder(new mojo.internal.DataViewWrapper(t, n.headerSize), o, { endpoint: e }).decodeStructInline(i.responseStruct.$.structSpec);
    if (!r) return this.onError(e, 'Received malformed response message');
    if (n.ordinal !== i.ordinal) return this.onError(e, 'Received malformed response message');
    if (i.useResultResponse) {
      const e = r.result;
      void 0 !== e.success ? i.resolve(e.success) : i.reject(e.failure);
    } else i.resolve(r);
  }
  onError(e, n = void 0) {
    this.cleanupAndFlushPendingResponses_(n), this.connectionErrorEventRouter_.dispatchErrorEvent();
  }
  cleanupAndFlushPendingResponses_(e = void 0) {
    this.endpoint_ && this.endpoint_.stop();
    for (const n of this.pendingResponses_.keys()) this.pendingResponses_.get(n).reject(new Error(e));
    this.pendingResponses_ = new Map();
  }
}, mojo.internal.interfaceSupport.InterfaceRemoteBaseWrapper = class {
  constructor(e) {
    this.remote_ = e;
  }
  bindNewPipeAndPassReceiver() {
    return this.remote_.bindNewPipeAndPassReceiver();
  }
  associateAndPassReceiver() {
    return this.remote_.associateAndPassReceiver();
  }
  isBound() {
    return null !== this.remote_.endpoint_;
  }
  close() {
    this.remote_.close();
  }
  flushForTesting() {
    return this.remote_.flushForTesting();
  }
}, mojo.internal.interfaceSupport.CallbackRouter = class {
  constructor() {
    this.removeCallbacks = new Map(), this.nextListenerId_ = 0;
  }
  getNextId() {
    return ++this.nextListenerId_;
  }
  removeListener(e) {
    return this.removeCallbacks.get(e)(), this.removeCallbacks.delete(e);
  }
}, mojo.internal.interfaceSupport.InterfaceCallbackReceiver = class {
  constructor(e) {
    this.listeners_ = new Map(), this.callbackRouter_ = e;
  }
  addListener(e) {
    const n = this.callbackRouter_.getNextId();
    return this.listeners_.set(n, e), this.callbackRouter_.removeCallbacks.set(n, (() => this.listeners_.delete(n))), n;
  }
  createReceiverHandler(e) {
    return e ? this.dispatchWithResponse_.bind(this) : this.dispatch_.bind(this);
  }
  dispatch_(e) {
    const n = Array.from(arguments);
    this.listeners_.forEach(((e) => e.apply(null, n)));
  }
  dispatchWithResponse_(e) {
    const n = Array.from(arguments), t = Array.from(this.listeners_.values()).map(((e) => e.apply(null, n)));
    let o;
    for (const e of t) if (void 0 !== e) {
      if (void 0 !== o) throw new Error('Multiple listeners attempted to reply to a message');
      o = e;
    }
    return o;
  }
}, mojo.internal.interfaceSupport.MessageHandler, mojo.internal.interfaceSupport.InterfaceReceiverHelperInternal = class {
  constructor(e) {
    this.endpoints_ = new Set(), this.remoteType_ = e, this.messageHandlers_ = new Map(), this.connectionErrorEventRouter_ = new mojo.internal.interfaceSupport.ConnectionErrorEventRouter();
  }
  registerHandler(e, n, t, o, i) {
    this.messageHandlers_.set(e, {
      paramStruct: n,
      responseStruct: t,
      handler: o,
      useResultResponse: i
    });
  }
  bindHandle(e) {
    e = mojo.internal.interfaceSupport.createEndpoint(e), this.endpoints_.add(e), e.start(this);
  }
  bindNewPipeAndPassRemote() {
    let e = new this.remoteType_();
    return this.bindHandle(e.$.bindNewPipeAndPassReceiver().handle), e;
  }
  associateAndPassRemote() {
    const { endpoint0: e, endpoint1: n } = mojo.internal.interfaceSupport.Endpoint.createAssociatedPair();
    return this.bindHandle(e), new this.remoteType_(n);
  }
  closeBindings() {
    for (const e of this.endpoints_) e.close();
    this.endpoints_.clear();
  }
  getConnectionErrorEventRouter() {
    return this.connectionErrorEventRouter_;
  }
  async flush() {
    for (let e of this.endpoints_) await e.flushForTesting();
  }
  onMessageReceived(e, n, t, o) {
    if (n.flags & mojo.internal.kMessageFlagIsResponse) throw new Error('Received unexpected response on interface receiver');
    const i = this.messageHandlers_.get(n.ordinal);
    if (!i) throw new Error('Received unknown message');
    const r = new mojo.internal.Decoder(new mojo.internal.DataViewWrapper(t, n.headerSize), o, { endpoint: e }).decodeStructInline(i.paramStruct.$.structSpec);
    if (!r) throw new Error('Received malformed message');
    const s = [];
    for (const e of i.paramStruct.$.structSpec.fields) {
      if (!mojo.internal.isNullableValueKindField(e)) {
        s.push(r[e.name]);
        continue;
      }
      const n = e.nullableValueKindProperties;
      n.isPrimary && s.push(r[n.originalFieldName]);
    }
    i.useResultResponse ? this.handleResultResponseMessage_(e, n, i, s) : this.handleResponseMessage_(e, n, i, s);
  }
  handleResultResponseMessage_(e, n, t, o) {
    try {
      let i = t.handler.apply(null, o);
      'object' == typeof i && 'Promise' == i.constructor.name || (i = Promise.resolve(i)), i.then(((o) => {
        e.send(n.ordinal, n.requestId, mojo.internal.kMessageFlagIsResponse, t.responseStruct, { result: { success: o } });
      })).catch(((o) => {
        e.send(n.ordinal, n.requestId, mojo.internal.kMessageFlagIsResponse, t.responseStruct, { result: { failure: o } });
      }));
    } catch (o) {
      e.send(n.ordinal, n.requestId, mojo.internal.kMessageFlagIsResponse, t.responseStruct, { result: { failure: o } });
    }
  }
  handleResponseMessage_(e, n, t, o) {
    let i = t.handler.apply(null, o);
    if (t.responseStruct) {
      if (void 0 === i) throw this.onError(e), new Error('Message expects a reply but its handler did not provide one.');
      'object' == typeof i && 'Promise' == i.constructor.name || (i = Promise.resolve(i)), i.then(((o) => {
        e.send(n.ordinal, n.requestId, mojo.internal.kMessageFlagIsResponse, t.responseStruct, o);
      })).catch((() => {
        this.onError(e);
      }));
    }
  }
  onError(e, n = void 0) {
    this.endpoints_.delete(e), e.close(), this.connectionErrorEventRouter_.dispatchErrorEvent();
  }
}, mojo.internal.interfaceSupport.InterfaceReceiverHelper = class {
  constructor(e) {
    this.helper_internal_ = e;
  }
  bindHandle(e) {
    this.helper_internal_.bindHandle(e);
  }
  bindNewPipeAndPassRemote() {
    return this.helper_internal_.bindNewPipeAndPassRemote();
  }
  associateAndPassRemote() {
    return this.helper_internal_.associateAndPassRemote();
  }
  close() {
    this.helper_internal_.closeBindings();
  }
  flush() {
    return this.helper_internal_.flush();
  }
}, mojo.internal.interfaceSupport.HandleReader = class {
  constructor(e) {
    this.handle_ = e, this.onRead = null, this.onError = () => {}, this.watcher_ = null;
  }
  isStopped() {
    return null === this.watcher_;
  }
  start() {
    this.watcher_ = this.handle_.watch({ readable: !0 }, this.read_.bind(this));
  }
  stop() {
    this.watcher_ && (this.watcher_.cancel(), this.watcher_ = null);
  }
  stopAndCloseHandle() {
    this.watcher_ && this.stop(), this.handle_.close();
  }
  read_(e) {
    for (;;) {
      if (!this.watcher_) return;
      const e = this.handle_.readMessage();
      if (e.result == Mojo.RESULT_SHOULD_WAIT) return;
      if (e.result == Mojo.RESULT_FAILED_PRECONDITION) return void this.onError();
      if (e.result != Mojo.RESULT_OK) throw new Error('Unexpected error on HandleReader: ' + e.result);
      this.onRead(e.buffer, e.handles);
    }
  }
};
const encoder = new TextEncoder();
const codecDecoder = new TextDecoder('utf-8', { fatal: true });
const MAX_COLLECTION_LENGTH = 16 * 1024 * 1024;
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
export function createMojomCommandApi(entries, namespace, transport = sendTypedRequest) {
  if (typeof transport !== 'function') throw new TypeError('expected typed Mojo transport');
  const root = Object.create(null);
  for (const [path, ordinal, request, response, interfaceName] of entries) {
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
    const endpoint = Object.freeze({
      interfaceName,
      ordinal,
      request,
      response
    });
    node[leaf] = async (...args) => {
      if (args.length > request.fields.length || request.fields.slice(args.length).some(([, , , type]) => !type?.nullable)) throw new TypeError('invalid Mojom argument count');
      const value = Object.fromEntries(request.fields.map(([name], index) => [name, args[index]]));
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
  6: 'resource_exhausted'
});
const mojoTypes = new WeakMap();
const nativeRemotes = new Map();
const responseTypes = new WeakMap();
function nativeType(schema) {
  if (typeof schema === 'string') {
    if (schema === 'bytes') return mojo.internal.Array(mojo.internal.Uint8, false);
    const names = {
      bool: 'Bool',
      string: 'String',
      u8: 'Uint8',
      i8: 'Int8',
      u16: 'Uint16',
      i16: 'Int16',
      u32: 'Uint32',
      i32: 'Int32',
      u64: 'Uint64',
      i64: 'Int64',
      f32: 'Float',
      f64: 'Double'
    };
    return mojo.internal[names[schema]];
  }
  if (schema.nullable) return nativeType(schema.nullable);
  if (schema.enum) return mojo.internal.Enum();
  if (schema.array) return mojo.internal.Array(nativeType(schema.array), !!schema.array.nullable);
  if (mojoTypes.has(schema)) return mojoTypes.get(schema);
  const type = {};
  mojoTypes.set(schema, type);
  const fields = [];
  for (const [name, offset, bit, field, presence] of schema.fields) {
    if (presence) {
      const primary = {
        isPrimary: true,
        originalFieldName: name,
        linkedValueFieldName: name + 'Value'
      };
      const secondary = {
        isPrimary: false,
        originalFieldName: name,
        linkedValueFieldName: name + 'Present'
      };
      fields.push(mojo.internal.StructField(name + 'Present', presence[0] - 8, presence[1], mojo.internal.Bool, false, false, 0, primary));
      const fallback = field.nullable?.enum?.[0] ?? (['u64', 'i64'].includes(field.nullable) ? 0n : field.nullable === 'bool' ? false : 0);
      fields.push(mojo.internal.StructField(name + 'Value', offset - 8, bit, nativeType(field.nullable), fallback, false, 0, secondary));
    } else {
      fields.push(mojo.internal.StructField(name, offset - 8, bit, nativeType(field), null, !!field?.nullable));
    }
  }
  mojo.internal.Struct(type, 'Tuffite typed parameters', fields, [[0, schema.size]]);
  return type;
}
function normalizeNative(schema, value) {
  if (value == null) return value;
  if (schema === 'bytes') return new Uint8Array(value);
  if (typeof schema !== 'object') return value;
  if (schema.nullable) return normalizeNative(schema.nullable, value);
  if (schema.array) return value.map((item) => normalizeNative(schema.array, item));
  if (schema.enum) return value;
  return Object.fromEntries(schema.fields.map(([name, , , type]) => [name, normalizeNative(type, value[name])]));
}
async function sendTypedRequest(endpoint, value) {
  if (typeof globalThis.Mojo?.bindInterface !== 'function') {
    throw new TuffiteError('transport_unavailable', 'Mojo is unavailable for this document');
  }
  let remote = nativeRemotes.get(endpoint.interfaceName);
  if (!remote) {
    const { handle0, handle1 } = Mojo.createMessagePipe();
    remote = new mojo.internal.interfaceSupport.InterfaceRemoteBase(null, handle0);
    Mojo.bindInterface(endpoint.interfaceName, handle1);
    remote.getConnectionErrorEventRouter().addListener(() => nativeRemotes.delete(endpoint.interfaceName));
    nativeRemotes.set(endpoint.interfaceName, remote);
  }
  let responseType = responseTypes.get(endpoint);
  if (!responseType) {
    responseType = nativeType({
      size: 32,
      fields: [
        [
          'status',
          8,
          0,
          'i32'
        ],
        [
          'value',
          16,
          0,
          { nullable: endpoint.response }
        ],
        [
          'error',
          24,
          0,
          { nullable: 'string' }
        ]
      ]
    });
    responseTypes.set(endpoint, responseType);
  }
  let reply;
  try {
    reply = await remote.sendMessage(endpoint.ordinal, nativeType(endpoint.request), responseType, endpoint.request.fields.map(([name]) => value[name]), false);
  } catch (cause) {
    throw new TuffiteError('transport_unavailable', cause?.message || 'Mojo interface disconnected', { cause });
  }
  if (reply.status !== 0) throw new TuffiteError(transportCodes[reply.status] ?? 'internal', reply.error ?? 'Mojo method failed', { cause: { status: reply.status } });
  if (!reply.value) throw new TuffiteError('decode_error', 'Missing typed Mojo response');
  const result = normalizeNative(endpoint.response, reply.value);
  encodeMojom(endpoint.response, result);
  return result;
}
function windowReference(info) {
  if (info && typeof info.id === 'bigint') {
    const id = info.id;
    Object.defineProperty(info, 'postMessage', { value: (...args) => globalThis.tuffite.postWindowMessage(id, ...args) });
  }
  return info;
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
  'DialogSaveFileRequest': {
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
    ], [
      'contents',
      16,
      0,
      'bytes'
    ]],
    'size': 24
  },
  'DialogSaveFileResponse': {
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
  'WindowSetConstraintsRequest': {
    'fields': [[
      'constraints',
      8,
      0,
      {
        'fields': [
          [
            'minWidth',
            8,
            0,
            'f64'
          ],
          [
            'minHeight',
            16,
            0,
            'f64'
          ],
          [
            'maxWidth',
            32,
            0,
            { 'nullable': 'f64' },
            [24, 0]
          ],
          [
            'maxHeight',
            40,
            0,
            { 'nullable': 'f64' },
            [24, 1]
          ]
        ],
        'size': 48
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
  'WindowSetConstraintsResponse': {
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
  },
  'WindowSizeConstraints': {
    'fields': [
      [
        'minWidth',
        8,
        0,
        'f64'
      ],
      [
        'minHeight',
        16,
        0,
        'f64'
      ],
      [
        'maxWidth',
        32,
        0,
        { 'nullable': 'f64' },
        [24, 0]
      ],
      [
        'maxHeight',
        40,
        0,
        { 'nullable': 'f64' },
        [24, 1]
      ]
    ],
    'size': 48
  }
};
export const mojomMethods = [
  [
    'dialog.openFile',
    0,
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
    'tuffite.wire.Dialog'
  ],
  [
    'dialog.openDirectory',
    1,
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
    'tuffite.wire.Dialog'
  ],
  [
    'dialog.saveFile',
    2,
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
      ], [
        'contents',
        16,
        0,
        'bytes'
      ]],
      'size': 24
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
    'tuffite.wire.Dialog'
  ],
  [
    'path.directories',
    0,
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
    'tuffite.wire.Path'
  ],
  [
    'path.resolve',
    1,
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
    'tuffite.wire.Path'
  ],
  [
    'remoting.capabilities',
    0,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.start',
    1,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.processSignal',
    2,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.takeSignals',
    3,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.sendInput',
    4,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.stop',
    5,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.stats',
    6,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.capture',
    7,
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
    'tuffite.wire.Remoting'
  ],
  [
    'remoting.injectInput',
    8,
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
    'tuffite.wire.Remoting'
  ],
  [
    'fs.readFile',
    0,
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
    'tuffite.wire.Fs'
  ],
  [
    'fs.writeFile',
    1,
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
    'tuffite.wire.Fs'
  ],
  [
    'fs.exists',
    2,
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
    'tuffite.wire.Fs'
  ],
  [
    'fs.createDir',
    3,
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
    'tuffite.wire.Fs'
  ],
  [
    'fs.removeFile',
    4,
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
    'tuffite.wire.Fs'
  ],
  [
    'storage.get',
    0,
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
    'tuffite.wire.Storage'
  ],
  [
    'storage.set',
    1,
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
    'tuffite.wire.Storage'
  ],
  [
    'storage.remove',
    2,
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
    'tuffite.wire.Storage'
  ],
  [
    'storage.clear',
    3,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [],
      'size': 8
    },
    'tuffite.wire.Storage'
  ],
  [
    'log.trace',
    0,
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
    'tuffite.wire.Log'
  ],
  [
    'log.debug',
    1,
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
    'tuffite.wire.Log'
  ],
  [
    'log.info',
    2,
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
    'tuffite.wire.Log'
  ],
  [
    'log.warn',
    3,
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
    'tuffite.wire.Log'
  ],
  [
    'log.error',
    4,
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
    'tuffite.wire.Log'
  ],
  [
    'net.request',
    0,
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
    'tuffite.wire.Net'
  ],
  [
    'utility.invoke',
    0,
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
    'tuffite.wire.Utility'
  ],
  [
    'utility.cancel',
    1,
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
    'tuffite.wire.Utility'
  ],
  [
    'window.current',
    0,
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
    'tuffite.wire.Window'
  ],
  [
    'window.list',
    1,
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
    'tuffite.wire.Window'
  ],
  [
    'window.create',
    2,
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
    'tuffite.wire.Window'
  ],
  [
    'window.focus',
    3,
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
    'tuffite.wire.Window'
  ],
  [
    'window.minimize',
    4,
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
    'tuffite.wire.Window'
  ],
  [
    'window.maximize',
    5,
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
    'tuffite.wire.Window'
  ],
  [
    'window.restore',
    6,
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
    'tuffite.wire.Window'
  ],
  [
    'window.setFullscreen',
    7,
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
    'tuffite.wire.Window'
  ],
  [
    'window.close',
    8,
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
    'tuffite.wire.Window'
  ],
  [
    'window.setTitle',
    9,
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
    'tuffite.wire.Window'
  ],
  [
    'window.getBounds',
    10,
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
    'tuffite.wire.Window'
  ],
  [
    'window.setBounds',
    11,
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
    'tuffite.wire.Window'
  ],
  [
    'window.setConstraints',
    12,
    {
      'fields': [[
        'constraints',
        8,
        0,
        {
          'fields': [
            [
              'minWidth',
              8,
              0,
              'f64'
            ],
            [
              'minHeight',
              16,
              0,
              'f64'
            ],
            [
              'maxWidth',
              32,
              0,
              { 'nullable': 'f64' },
              [24, 0]
            ],
            [
              'maxHeight',
              40,
              0,
              { 'nullable': 'f64' },
              [24, 1]
            ]
          ],
          'size': 48
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
    'tuffite.wire.Window'
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
export function createTuffiteApi(transport) {
  return createMojomCommandApi(mojomMethods, 'tuffite', transport);
}
export const { dialog, fs, log, net, path, remoting, storage, utility, window } = createTuffiteApi();

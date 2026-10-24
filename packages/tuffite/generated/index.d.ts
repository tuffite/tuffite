// Generated from Mojom v0. Do not edit.
export type TuffiteErrorCode = 'invalid_argument' | 'incompatible_abi' | 'invalid_handle' | 'unavailable' | 'resource_exhausted' | 'internal' | 'transport_unavailable' | 'invalid_request' | 'unknown_command' | 'handler_failed' | 'decode_error' | 'protocol_mismatch' | 'schema_mismatch' | 'command_failed' | (string & {});
export class TuffiteError extends Error {
  readonly code: TuffiteErrorCode;
  readonly requestId?: bigint;
  readonly cause?: unknown;
  constructor(code: TuffiteErrorCode, message: string, options?: Readonly<{
    requestId?: bigint;
    cause?: unknown;
  }>);
}
export interface ChannelEvent extends Event {
  readonly detail: Readonly<{
    channel: string;
    data: Uint8Array;
  }>;
}
export type ExplorerMessageEvent<T = unknown> = MessageEvent<T>;
export interface ExplorerOpenEvent extends Event {
  readonly detail: Readonly<{
    url: string;
    disposition: string;
    userGesture: boolean;
    from: string;
  }>;
}
export interface ExplorerNewWindowEvent extends Event {
  readonly detail: import('tuffite/explorer').ExplorerNewWindowController;
}
export interface ExplorerDialogController {
  readonly signal: AbortSignal;
  readonly type: 'alert' | 'confirm' | 'prompt' | 'beforeunload';
  readonly message: string;
  readonly defaultValue: string;
  readonly isReload: boolean;
  readonly from: string;
  accept(promptText?: string): Promise<boolean>;
  dismiss(): Promise<boolean>;
}
export interface ExplorerDialogEvent extends Event {
  readonly detail: ExplorerDialogController;
}
export interface NativeBridge extends EventTarget {
  addEventListener(type: 'message' | 'messageerror', listener: (event: MessageEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: 'message' | 'messageerror', listener: (event: MessageEvent) => void, options?: boolean | EventListenerOptions): void;
  addEventListener(type: string, listener: EventListenerOrEventListenerObject | null, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: string, listener: EventListenerOrEventListenerObject | null, options?: boolean | EventListenerOptions): void;
  invoke(channel: string, payload: Uint8Array): Promise<Uint8Array>;
  postMessage(message: unknown, options?: import('tuffite/explorer').ExplorerWindowPostMessageOptions): void;
  postMessage(message: unknown, targetOrigin: string, transfer?: import('tuffite/explorer').ExplorerTransferable[]): void;
  addEventListener(type: 'channel', listener: (event: ChannelEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: 'channel', listener: (event: ChannelEvent) => void, options?: boolean | EventListenerOptions): void;
  addEventListener(type: 'message', listener: (event: ExplorerMessageEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: 'message', listener: (event: ExplorerMessageEvent) => void, options?: boolean | EventListenerOptions): void;
  addEventListener(type: 'navigation', listener: (event: ExplorerOpenEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: 'navigation', listener: (event: ExplorerOpenEvent) => void, options?: boolean | EventListenerOptions): void;
  addEventListener(type: 'newwindow', listener: (event: ExplorerNewWindowEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: 'newwindow', listener: (event: ExplorerNewWindowEvent) => void, options?: boolean | EventListenerOptions): void;
  addEventListener(type: 'dialog', listener: (event: ExplorerDialogEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: 'dialog', listener: (event: ExplorerDialogEvent) => void, options?: boolean | EventListenerOptions): void;
}
export type MojomType = 'bool' | 'i8' | 'u8' | 'i16' | 'u16' | 'i32' | 'u32' | 'i64' | 'u64' | 'f32' | 'f64' | 'string' | 'bytes' | MojomStructSchema | {
  readonly enum: readonly number[];
} | {
  readonly nullable: MojomType;
} | {
  readonly array: MojomType;
  readonly width: number;
  readonly boolean: boolean;
};
export interface MojomStructSchema {
  readonly size: number;
  readonly fields: readonly (readonly [name: string, offset: number, bit: number, type: MojomType, presence?: readonly [offset: number, bit: number]])[];
}
export function encodeMojom(schema: MojomStructSchema, value: object): Uint8Array;
export function decodeMojom(schema: MojomStructSchema, bytes: Uint8Array): Record<string, unknown>;
export type MojomTransport = (channel: string, commandId: bigint, schemaFingerprint: bigint, payload: Uint8Array) => Promise<Uint8Array>;
export type MojomCommandEntry = readonly [path: string, id: string, fingerprint: string, request: MojomStructSchema, response: MojomStructSchema, channel?: string];
export function createMojomCommandApi(entries: readonly MojomCommandEntry[], namespace: string, transport?: MojomTransport): object;
export const invokeFrameworkMojom: MojomTransport;
export function invokeBinary(channel: string, commandId: bigint, schemaFingerprint: bigint, payload?: Uint8Array): Promise<Uint8Array>;
export interface WindowInfo {
  postMessage(message: unknown, options?: import('tuffite/explorer').ExplorerWindowPostMessageOptions): void;
  postMessage(message: unknown, targetOrigin: string, transfer?: import('tuffite/explorer').ExplorerTransferable[]): void;
}
export declare const mojomTypes: Readonly<Record<string, MojomStructSchema>>;
export declare const mojomMethods: readonly MojomCommandEntry[];
export enum BaseDirectory {
  Resource = 0,
  Data = 1,
  Cache = 2,
  Temp = 3
}
export interface DialogOpenDirectoryRequest {
  'options'?: OpenDirectoryDialogOptions | null;
}
export interface DialogOpenDirectoryResponse {
  'result'?: string | null;
}
export interface DialogOpenFileRequest {
  'options'?: OpenFileDialogOptions | null;
}
export interface DialogOpenFileResponse {
  'result'?: string | null;
}
export interface FsCreateDirRequest {
  'base': BaseDirectory;
  'path': string;
}
export interface FsCreateDirResponse {}
export interface FsExistsRequest {
  'base': BaseDirectory;
  'path': string;
}
export interface FsExistsResponse {
  'result': boolean;
}
export interface FsReadFileRequest {
  'base': BaseDirectory;
  'path': string;
}
export interface FsReadFileResponse {
  'result': Uint8Array;
}
export interface FsRemoveFileRequest {
  'base': BaseDirectory;
  'path': string;
}
export interface FsRemoveFileResponse {}
export interface FsWriteFileRequest {
  'base': BaseDirectory;
  'path': string;
  'contents': Uint8Array;
}
export interface FsWriteFileResponse {}
export interface LogDebugRequest {
  'message': string;
}
export interface LogDebugResponse {}
export interface LogErrorRequest {
  'message': string;
}
export interface LogErrorResponse {}
export interface LogInfoRequest {
  'message': string;
}
export interface LogInfoResponse {}
export interface LogTraceRequest {
  'message': string;
}
export interface LogTraceResponse {}
export interface LogWarnRequest {
  'message': string;
}
export interface LogWarnResponse {}
export interface NetRequestRequest {
  'url': string;
}
export interface NetRequestResponse {
  'result': NetworkResponse;
}
export interface NetworkResponse {
  'status': number;
  'body': Uint8Array;
}
export interface OpenDirectoryDialogOptions {
  'title'?: string | null;
}
export interface OpenFileDialogOptions {
  'title'?: string | null;
  'defaultPath'?: string | null;
  'extensions'?: string[][] | null;
}
export interface PathDirectories {
  'resource': string;
  'data': string;
  'cache': string;
  'temp': string;
}
export interface PathDirectoriesRequest {}
export interface PathDirectoriesResponse {
  'result': PathDirectories;
}
export interface PathResolveRequest {
  'base': BaseDirectory;
  'relative'?: string | null;
}
export interface PathResolveResponse {
  'result': string;
}
export interface RemotingCapabilities {
  'available': boolean;
  'protocol': string;
  'videoCodecs': string[];
  'input': boolean;
  'clipboard': boolean;
  'reconnect': boolean;
  'snapshot': boolean;
}
export interface RemotingCapabilitiesRequest {}
export interface RemotingCapabilitiesResponse {
  'result': RemotingCapabilities;
}
export interface RemotingCaptureRequest {
  'screenId'?: bigint | null;
}
export interface RemotingCaptureResponse {
  'result': RemotingSnapshot;
}
export interface RemotingIceCandidate {
  'mid': string;
  'mLineIndex': number;
  'candidate': string;
}
export interface RemotingIceServer {
  'urls': string[];
  'username'?: string | null;
  'credential'?: string | null;
}
export interface RemotingInjectInputRequest {
  'event': RemotingInputEvent;
}
export interface RemotingInjectInputResponse {}
export interface RemotingInputEvent {
  'kind': string;
  'pressed'?: boolean | null;
  'usbKeycode'?: number | null;
  'text'?: string | null;
  'x'?: number | null;
  'y'?: number | null;
  'button'?: string | null;
  'buttonDown'?: boolean | null;
  'wheelDeltaX'?: number | null;
  'wheelDeltaY'?: number | null;
  'wheelTicksX'?: number | null;
  'wheelTicksY'?: number | null;
  'clipboardMimeType'?: string | null;
  'clipboardData'?: Uint8Array | null;
}
export interface RemotingProcessSignalRequest {
  'id': bigint;
  'signal': RemotingSignal;
}
export interface RemotingProcessSignalResponse {}
export interface RemotingSendInputRequest {
  'id': bigint;
  'event': RemotingInputEvent;
}
export interface RemotingSendInputResponse {}
export interface RemotingSession {
  'id': bigint;
  'state': string;
}
export interface RemotingSessionDescription {
  'type': string;
  'sdp': string;
}
export interface RemotingSignal {
  'sequence': bigint;
  'description'?: RemotingSessionDescription | null;
  'candidates': RemotingIceCandidate[];
}
export interface RemotingSnapshot {
  'width': number;
  'height': number;
  'mimeType': string;
  'data': Uint8Array;
}
export interface RemotingStartOptions {
  'authKey': string;
  'iceServers': RemotingIceServer[];
  'maxBitrateKbps'?: number | null;
  'targetFramerate'?: number | null;
  'screenId'?: bigint | null;
}
export interface RemotingStartRequest {
  'options': RemotingStartOptions;
}
export interface RemotingStartResponse {
  'result': RemotingSession;
}
export interface RemotingStats {
  'state': string;
  'route': string;
  'transport': string;
  'latencyMs': number;
  'jitterMs': number;
  'availableBandwidthKbps': number;
  'packetLossPercent': number;
}
export interface RemotingStatsRequest {
  'id': bigint;
}
export interface RemotingStatsResponse {
  'result': RemotingStats;
}
export interface RemotingStopRequest {
  'id': bigint;
}
export interface RemotingStopResponse {}
export interface RemotingTakeSignalsRequest {
  'id': bigint;
  'afterSequence'?: bigint | null;
}
export interface RemotingTakeSignalsResponse {
  'result': RemotingSignal[];
}
export interface StorageClearRequest {}
export interface StorageClearResponse {}
export interface StorageGetRequest {
  'key': string;
}
export interface StorageGetResponse {
  'result'?: Uint8Array | null;
}
export interface StorageRemoveRequest {
  'key': string;
}
export interface StorageRemoveResponse {}
export interface StorageSetRequest {
  'key': string;
  'value': Uint8Array;
}
export interface StorageSetResponse {}
export interface UtilityCancelRequest {
  'name': string;
  'requestId': bigint;
}
export interface UtilityCancelResponse {}
export interface UtilityInvokeRequest {
  'name': string;
  'requestId': bigint;
  'payload'?: Uint8Array | null;
}
export interface UtilityInvokeResponse {
  'result': Uint8Array;
}
export interface WindowBounds {
  'x': number;
  'y': number;
  'width': number;
  'height': number;
}
export interface WindowCloseRequest {
  'id'?: bigint | null;
}
export interface WindowCloseResponse {}
export interface WindowCreateRequest {
  'options'?: WindowOptions | null;
}
export interface WindowCreateResponse {
  'result': WindowInfo;
}
export interface WindowCurrentRequest {}
export interface WindowCurrentResponse {
  'result': WindowInfo;
}
export interface WindowFocusRequest {
  'id'?: bigint | null;
}
export interface WindowFocusResponse {
  'result': WindowInfo;
}
export interface WindowGetBoundsRequest {
  'id'?: bigint | null;
}
export interface WindowGetBoundsResponse {
  'result': WindowBounds;
}
export interface WindowInfo {
  'id': bigint;
  'label': string;
  'title': string;
  'url': string;
  'bounds': WindowBounds;
  'focused': boolean;
  'minimized': boolean;
  'maximized': boolean;
  'fullscreen': boolean;
}
export interface WindowListRequest {}
export interface WindowListResponse {
  'result': WindowInfo[];
}
export interface WindowMaximizeRequest {
  'id'?: bigint | null;
}
export interface WindowMaximizeResponse {
  'result': WindowInfo;
}
export interface WindowMinimizeRequest {
  'id'?: bigint | null;
}
export interface WindowMinimizeResponse {
  'result': WindowInfo;
}
export interface WindowOptions {
  'label'?: string | null;
  'url'?: string | null;
  'title'?: string | null;
  'bounds'?: WindowBounds | null;
  'focused'?: boolean | null;
}
export interface WindowRestoreRequest {
  'id'?: bigint | null;
}
export interface WindowRestoreResponse {
  'result': WindowInfo;
}
export interface WindowSetBoundsRequest {
  'bounds': WindowBounds;
  'id'?: bigint | null;
}
export interface WindowSetBoundsResponse {
  'result': WindowInfo;
}
export interface WindowSetFullscreenRequest {
  'fullscreen': boolean;
  'id'?: bigint | null;
}
export interface WindowSetFullscreenResponse {
  'result': WindowInfo;
}
export interface WindowSetTitleRequest {
  'title': string;
  'id'?: bigint | null;
}
export interface WindowSetTitleResponse {
  'result': WindowInfo;
}
export interface TuffiteApi {
  readonly 'dialog': typeof dialog;
  readonly 'fs': typeof fs;
  readonly 'log': typeof log;
  readonly 'net': typeof net;
  readonly 'path': typeof path;
  readonly 'remoting': typeof remoting;
  readonly 'storage': typeof storage;
  readonly 'utility': typeof utility;
  readonly 'window': typeof window;
}
export declare function createTuffiteApi(transport?: MojomTransport): TuffiteApi;
export declare const dialog: {
  readonly 'openFile': (arg0?: OpenFileDialogOptions | null) => Promise<string | null>;
  readonly 'openDirectory': (arg0?: OpenDirectoryDialogOptions | null) => Promise<string | null>;
};
export declare const fs: {
  readonly 'readFile': (arg0: BaseDirectory, arg1: string) => Promise<Uint8Array>;
  readonly 'writeFile': (arg0: BaseDirectory, arg1: string, arg2: Uint8Array) => Promise<void>;
  readonly 'exists': (arg0: BaseDirectory, arg1: string) => Promise<boolean>;
  readonly 'createDir': (arg0: BaseDirectory, arg1: string) => Promise<void>;
  readonly 'removeFile': (arg0: BaseDirectory, arg1: string) => Promise<void>;
};
export declare const log: {
  readonly 'trace': (arg0: string) => Promise<void>;
  readonly 'debug': (arg0: string) => Promise<void>;
  readonly 'info': (arg0: string) => Promise<void>;
  readonly 'warn': (arg0: string) => Promise<void>;
  readonly 'error': (arg0: string) => Promise<void>;
};
export declare const net: {
  readonly 'request': (arg0: string) => Promise<NetworkResponse>;
};
export declare const path: {
  readonly 'directories': () => Promise<PathDirectories>;
  readonly 'resolve': (arg0: BaseDirectory, arg1?: string | null) => Promise<string>;
};
export declare const remoting: {
  readonly 'capabilities': () => Promise<RemotingCapabilities>;
  readonly 'start': (arg0: RemotingStartOptions) => Promise<RemotingSession>;
  readonly 'processSignal': (arg0: bigint, arg1: RemotingSignal) => Promise<void>;
  readonly 'takeSignals': (arg0: bigint, arg1?: bigint | null) => Promise<RemotingSignal[]>;
  readonly 'sendInput': (arg0: bigint, arg1: RemotingInputEvent) => Promise<void>;
  readonly 'stop': (arg0: bigint) => Promise<void>;
  readonly 'stats': (arg0: bigint) => Promise<RemotingStats>;
  readonly 'capture': (arg0?: bigint | null) => Promise<RemotingSnapshot>;
  readonly 'injectInput': (arg0: RemotingInputEvent) => Promise<void>;
};
export declare const storage: {
  readonly 'get': (arg0: string) => Promise<Uint8Array | null>;
  readonly 'set': (arg0: string, arg1: Uint8Array) => Promise<void>;
  readonly 'remove': (arg0: string) => Promise<void>;
  readonly 'clear': () => Promise<void>;
};
export declare const utility: {
  readonly 'invoke': (arg0: string, arg1: bigint, arg2?: Uint8Array | null) => Promise<Uint8Array>;
  readonly 'cancel': (arg0: string, arg1: bigint) => Promise<void>;
};
export declare const window: {
  readonly 'current': () => Promise<WindowInfo>;
  readonly 'list': () => Promise<WindowInfo[]>;
  readonly 'create': (arg0?: WindowOptions | null) => Promise<WindowInfo>;
  readonly 'focus': (arg0?: bigint | null) => Promise<WindowInfo>;
  readonly 'minimize': (arg0?: bigint | null) => Promise<WindowInfo>;
  readonly 'maximize': (arg0?: bigint | null) => Promise<WindowInfo>;
  readonly 'restore': (arg0?: bigint | null) => Promise<WindowInfo>;
  readonly 'setFullscreen': (arg0: boolean, arg1?: bigint | null) => Promise<WindowInfo>;
  readonly 'close': (arg0?: bigint | null) => Promise<void>;
  readonly 'setTitle': (arg0: string, arg1?: bigint | null) => Promise<WindowInfo>;
  readonly 'getBounds': (arg0?: bigint | null) => Promise<WindowBounds>;
  readonly 'setBounds': (arg0: WindowBounds, arg1?: bigint | null) => Promise<WindowInfo>;
};

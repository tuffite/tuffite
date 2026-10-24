
export type TuffiteErrorCode =
  | "invalid_argument"
  | "incompatible_abi"
  | "invalid_handle"
  | "unavailable"
  | "resource_exhausted"
  | "internal"
  | "transport_unavailable"
  | "invalid_request"
  | "unknown_command"
  | "handler_failed"
  | "decode_error"
  | "protocol_mismatch"
  | "schema_mismatch"
  | "command_failed"
  | (string & {});

export class TuffiteError extends Error {
  readonly code: TuffiteErrorCode;
  readonly requestId?: bigint;
  readonly cause?: unknown;
  constructor(
    code: TuffiteErrorCode,
    message: string,
    options?: Readonly<{ requestId?: bigint; cause?: unknown }>,
  );
}

export interface ChannelEvent extends Event {
  readonly detail: Readonly<{ channel: string; data: Uint8Array }>;
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
  readonly detail: import("tuffite/explorer").ExplorerNewWindowController;
}

export interface ExplorerDialogController {
  readonly signal: AbortSignal;
  readonly type: "alert" | "confirm" | "prompt" | "beforeunload";
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
  addEventListener(type: "message" | "messageerror", listener: (event: MessageEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: "message" | "messageerror", listener: (event: MessageEvent) => void, options?: boolean | EventListenerOptions): void;
  addEventListener(type: string, listener: EventListenerOrEventListenerObject | null, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: string, listener: EventListenerOrEventListenerObject | null, options?: boolean | EventListenerOptions): void;
  invoke(channel: string, payload: Uint8Array): Promise<Uint8Array>;
  postMessage(message: unknown, options?: import("tuffite/explorer").ExplorerWindowPostMessageOptions): void;
  postMessage(message: unknown, targetOrigin: string, transfer?: import("tuffite/explorer").ExplorerTransferable[]): void;
  addEventListener(
    type: "channel",
    listener: (event: ChannelEvent) => void,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener(
    type: "channel",
    listener: (event: ChannelEvent) => void,
    options?: boolean | EventListenerOptions,
  ): void;
  addEventListener(
    type: "message",
    listener: (event: ExplorerMessageEvent) => void,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener(
    type: "message",
    listener: (event: ExplorerMessageEvent) => void,
    options?: boolean | EventListenerOptions,
  ): void;
  addEventListener(
    type: "navigation",
    listener: (event: ExplorerOpenEvent) => void,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener(
    type: "navigation",
    listener: (event: ExplorerOpenEvent) => void,
    options?: boolean | EventListenerOptions,
  ): void;
  addEventListener(type: "newwindow", listener: (event: ExplorerNewWindowEvent) => void, options?: boolean | AddEventListenerOptions): void;
  removeEventListener(type: "newwindow", listener: (event: ExplorerNewWindowEvent) => void, options?: boolean | EventListenerOptions): void;
  addEventListener(
    type: "dialog",
    listener: (event: ExplorerDialogEvent) => void,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener(
    type: "dialog",
    listener: (event: ExplorerDialogEvent) => void,
    options?: boolean | EventListenerOptions,
  ): void;
}

/** Handle-free Mojom v0 struct layout emitted by the native tuff compiler. */
export type MojomType = "bool" | "i8" | "u8" | "i16" | "u16" | "i32" | "u32" |
  "i64" | "u64" | "f32" | "f64" | "string" | "bytes" | MojomStructSchema |
  {readonly enum:readonly number[]} | {readonly nullable:MojomType} |
  {readonly array:MojomType; readonly width:number; readonly boolean:boolean};
export interface MojomStructSchema {
  readonly size:number;
  readonly fields:readonly (readonly [name:string, offset:number, bit:number, type:MojomType,
    presence?:readonly [offset:number,bit:number]])[];
}
export function encodeMojom(schema: MojomStructSchema, value: object): Uint8Array;
export function decodeMojom(schema: MojomStructSchema, bytes: Uint8Array): Record<string, unknown>;
/** Exchanges Mojom struct payloads; the default invokeBinary adds the ShellAPI envelope. */
export type MojomTransport = (
  channel: string, commandId: bigint, schemaFingerprint: bigint, payload: Uint8Array,
) => Promise<Uint8Array>;
export type MojomCommandEntry = readonly [path: string, id: string, fingerprint: string,
  request: MojomStructSchema, response: MojomStructSchema, channel?: string];

export function createMojomCommandApi(
  entries: readonly MojomCommandEntry[],
  namespace: string,
  transport?: MojomTransport,
): object;

export const invokeFrameworkMojom: MojomTransport;

export function invokeBinary(
  channel: string,
  commandId: bigint,
  schemaFingerprint: bigint,
  payload?: Uint8Array,
): Promise<Uint8Array>;

// Window snapshots expose messaging outside the Shell codec. They are not transferable.
export interface WindowInfo {
  postMessage(message: unknown, options?: import("tuffite/explorer").ExplorerWindowPostMessageOptions): void;
  postMessage(message: unknown, targetOrigin: string, transfer?: import("tuffite/explorer").ExplorerTransferable[]): void;
}

export type Codec = Readonly<Record<string, unknown>>;

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
  readonly detail: import("./explorer.js").ExplorerNewWindowController;
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
  postMessage(message: unknown, options?: import("./explorer").ExplorerWindowPostMessageOptions): void;
  postMessage(message: unknown, targetOrigin: string, transfer?: import("./explorer").ExplorerTransferable[]): void;
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

export type CommandEntry = readonly [
  path: string,
  id: string,
  schemaFingerprint: string,
  request: readonly Codec[],
  response: Codec,
];

export type ConstantEntry = readonly [path: string, value: unknown];

export function invokeBinary(
  channel: string,
  commandId: bigint,
  schemaFingerprint: bigint,
  payload?: Uint8Array,
): Promise<Uint8Array>;

export function createCommandApi(
  entries: readonly (readonly [path: string, id: bigint])[],
  typed?: boolean,
  rootTarget?: object,
  channelPrefix?: string,
): object;

export function createTypedCommandApi(
  entries: readonly CommandEntry[],
  constants?: readonly ConstantEntry[],
  namespace?: string,
): object;

export type FrameworkEntry = readonly [
  path: string,
  request: readonly Codec[],
  response: Codec,
];

export function createFrameworkApi(
  entries: readonly FrameworkEntry[],
  constants?: readonly ConstantEntry[],
): object;

// Window snapshots expose messaging outside the Shell codec. They are not transferable.
declare module "./generated/window" {
  interface WindowInfo {
    postMessage(message: unknown, options?: import("./explorer").ExplorerWindowPostMessageOptions): void;
    postMessage(message: unknown, targetOrigin: string, transfer?: import("./explorer").ExplorerTransferable[]): void;
  }
}

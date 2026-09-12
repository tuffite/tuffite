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

export interface ExplorerMessageEvent<T = unknown> extends Event {
  readonly detail: Readonly<{ data: T; from: string }>;
}

export interface ExplorerOpenEvent extends Event {
  readonly detail: Readonly<{
    url: string;
    disposition: string;
    userGesture: boolean;
    from: string;
  }>;
}

export interface ExplorerDialogController {
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

export type ExplorerSurfaceRef = string & {
  readonly __tuffiteExplorerSurfaceRef: unique symbol;
};
export type ExplorerMessageTarget = "parent" | "top" | ExplorerSurfaceRef;

export interface NativeBridge extends EventTarget {
  invoke(channel: string, payload: Uint8Array): Promise<Uint8Array>;
  postMessage(
    data: unknown,
    options?: Readonly<{ to?: ExplorerMessageTarget }>,
  ): Promise<boolean>;
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
    type: "open" | "popup" | "newwindow" | "navigation",
    listener: (event: ExplorerOpenEvent) => void,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener(
    type: "open" | "popup" | "newwindow" | "navigation",
    listener: (event: ExplorerOpenEvent) => void,
    options?: boolean | EventListenerOptions,
  ): void;
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

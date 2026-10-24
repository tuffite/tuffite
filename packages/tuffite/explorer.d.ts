export type ExplorerNavigationPolicy = "emit" | "allow";
export type ExplorerDialogPolicy = "emit" | "suppress";
export type ExplorerJavaScriptWorld = "isolated" | "main";
export type ExplorerPartitionMode = "shared" | "persist" | "memory";

export interface ExplorerNavigationState {
  readonly url: string;
  readonly title: string;
  readonly documentId: string;
  readonly canGoBack: boolean;
  readonly canGoForward: boolean;
  readonly isLoading: boolean;
  readonly zoomFactor: number;
  readonly audioMuted: boolean;
  readonly audible: boolean;
  readonly crashed: boolean;
  readonly fullscreen: boolean;
  readonly pictureInPicture: "none" | "video" | "document";
  readonly favicons: readonly ExplorerFavicon[];
  readonly rendererProcess: Readonly<{
    id: number;
    ownerId: number;
    isolated: boolean;
  }>;
  readonly storagePartition: Readonly<{
    domain: string;
    name: string;
    inMemory: boolean;
    isDefault: boolean;
    mode: ExplorerPartitionMode;
    shared: boolean;
    inherited: boolean;
    requested: string;
    createdDomain: string;
    createdName: string;
    createdInMemory: boolean;
    createdIsDefault: boolean;
  }>;
}

export interface ExplorerLoadErrorState extends ExplorerNavigationState {
  readonly errorCode: number;
  readonly errorDescription: string;
}

export interface ExplorerFavicon {
  readonly url: string;
  /** Chromium FaviconIconType: 1 favicon, 2 touch, 3 precomposed touch. */
  readonly type: number;
  readonly sizes: readonly Readonly<{width: number; height: number}>[];
  readonly isDefault: boolean;
}

export interface ExplorerFindOptions {
  readonly forward?: boolean;
  readonly matchCase?: boolean;
  readonly findNext?: boolean;
}

export interface ExplorerFindResult {
  readonly requestId: number;
  readonly matches: number;
  readonly activeMatchOrdinal: number;
  readonly finalUpdate: boolean;
  readonly selectionRect: Readonly<{x: number; y: number; width: number; height: number}>;
}

export interface ExplorerOpenDetail {
  readonly url: string;
  readonly disposition: string;
  readonly userGesture: boolean;
  readonly from: string;
}

export interface ExplorerNewWindowController extends ExplorerOpenDetail {
  readonly signal: AbortSignal;
  /** Adopts the original pending page into a connected Explorer. */
  attach(target: HTMLExplorerElement): Promise<boolean>;
  discard(): Promise<boolean>;
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

export type ExplorerHandleState = "pending" | "attaching" | "transferred" | "closed";

/** Exclusive native page ownership; no serializable credential is exposed. */
export interface ExplorerHandle extends EventTarget {
  readonly state: ExplorerHandleState;
  close(): Promise<void>;
  onstatechange: ((event: Event) => void) | null;
}

export interface ExplorerEventMap {
  download: CustomEvent<ExplorerDownload>;
  attached: Event;
  detached: Event;
  loadstart: CustomEvent<ExplorerNavigationState>;
  loadstop: CustomEvent<ExplorerNavigationState>;
  loadcommit: CustomEvent<ExplorerNavigationState>;
  loaderror: CustomEvent<ExplorerLoadErrorState>;
  findresult: CustomEvent<ExplorerFindResult>;
  titlechange: CustomEvent<ExplorerNavigationState>;
  fullscreenchange: CustomEvent<{readonly entered: boolean}>;
  pictureinpicturechange: CustomEvent<{readonly type: "video" | "document"; readonly entered: boolean}>;
  zoomchange: CustomEvent<ExplorerNavigationState & {readonly oldFactor: number; readonly newFactor: number}>;
  audiostatechange: CustomEvent<ExplorerNavigationState>;
  faviconchange: CustomEvent<ExplorerNavigationState>;
  renderprocessgone: CustomEvent<ExplorerNavigationState & {readonly reason: "normal" | "abnormal" | "killed" | "crashed" | "oom" | "evicted" | "launch-failed"; readonly terminationStatus: number}>;
  newwindow: CustomEvent<ExplorerNewWindowController>;
  navigation: CustomEvent<ExplorerOpenDetail>;
  dialog: CustomEvent<ExplorerDialogController>;
}

export interface HTMLExplorerElement extends HTMLElement {
  src: string;
  allow: string;
  /** Empty selects the parent's partition; `shared` is not an element value. */
  partition: string;
  /** Fixed at creation. Isolated keeps window.parent/top within this Explorer's frame tree. */
  isolated: boolean;
  navigation: ExplorerNavigationPolicy;
  dialogs: ExplorerDialogPolicy;
  contextMenuEnabled: boolean;
  detach(): Promise<ExplorerHandle>;
  attach(handle: ExplorerHandle): Promise<void>;
  ondetached: ((event: Event) => void) | null;
  onattached: ((event: Event) => void) | null;
  onloadstart: HTMLElement["onloadstart"];
  onloadstop: ((event: ExplorerEventMap["loadstop"]) => void) | null;
  onloadcommit: ((event: ExplorerEventMap["loadcommit"]) => void) | null;
  onloaderror: ((event: ExplorerEventMap["loaderror"]) => void) | null;
  ontitlechange: ((event: ExplorerEventMap["titlechange"]) => void) | null;
  ondownload: ((event: ExplorerEventMap["download"]) => void) | null;
  onfindresult: ((event: ExplorerEventMap["findresult"]) => void) | null;
  onzoomchange: ((event: ExplorerEventMap["zoomchange"]) => void) | null;
  onaudiostatechange: ((event: ExplorerEventMap["audiostatechange"]) => void) | null;
  onfaviconchange: ((event: ExplorerEventMap["faviconchange"]) => void) | null;
  onrenderprocessgone: ((event: ExplorerEventMap["renderprocessgone"]) => void) | null;
  onfullscreenchange: HTMLElement["onfullscreenchange"];
  onpictureinpicturechange: ((event: ExplorerEventMap["pictureinpicturechange"]) => void) | null;
  onnavigation: ((event: ExplorerEventMap["navigation"]) => void) | null;
  onnewwindow: ((event: ExplorerEventMap["newwindow"]) => void) | null;
  ondialog: ((event: ExplorerEventMap["dialog"]) => void) | null;
  /** Starts navigation; completion is reported by load events. */
  load(url: string): void;
  download(url: string): Promise<boolean>;
  getDownloads(): Promise<ExplorerDownload[]>;
  cancelDownload(id: number): Promise<boolean>;
  /** Base64 bytes of an owned completed download, bounded to 64 MiB. */
  readDownload(id: number): Promise<string>;
  /** A compositor PNG data URL, or an empty string when unavailable. */
  capturePreview(): Promise<string>;
  back(): Promise<boolean>;
  forward(): Promise<boolean>;
  go(offset: number): Promise<boolean>;
  reload(options?: Readonly<{ bypassCache?: boolean }>): Promise<boolean>;
  stop(): Promise<boolean>;
  getNavigationState(): Promise<ExplorerNavigationState>;
  /** Returns an accepted request ID; asynchronous updates arrive via findresult. */
  findInPage(text: string, options?: ExplorerFindOptions): Promise<number>;
  stopFindInPage(action?: "clear" | "keep" | "activate"): Promise<boolean>;
  /** Native per-Explorer page zoom, from 0.25 through 5.0; survives navigation/transfer. */
  setZoomFactor(factor: number): Promise<boolean>;
  getZoomFactor(): Promise<number>;
  /** Mutes this WebContents without changing media element properties. */
  setAudioMuted(muted: boolean): Promise<boolean>;
  /** The caller must validate the JSON-representable result at runtime. */
  executeJavaScript(
    source: string,
    options?: Readonly<{
      world?: ExplorerJavaScriptWorld;
      expectedDocumentId?: string;
    }>,
  ): Promise<unknown>;
  readonly contentWindow: Window | null;
  postMessage(message: unknown, options?: ExplorerWindowPostMessageOptions): void;
  postMessage(message: unknown, targetOrigin: string, transfer?: ExplorerTransferable[]): void;
  addEventListener<K extends keyof ExplorerEventMap>(
    type: K,
    listener: (this: HTMLExplorerElement, event: ExplorerEventMap[K]) => void,
    options?: boolean | AddEventListenerOptions,
  ): void;
  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject | null,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener<K extends keyof ExplorerEventMap>(
    type: K,
    listener: (this: HTMLExplorerElement, event: ExplorerEventMap[K]) => void,
    options?: boolean | EventListenerOptions,
  ): void;
  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject | null,
    options?: boolean | EventListenerOptions,
  ): void;
}

export type ExplorerTransferable = Transferable | ExplorerHandle;
export type ExplorerWindowPostMessageOptions = Omit<WindowPostMessageOptions, "transfer"> & {
  transfer?: ExplorerTransferable[];
};

declare global {
  function postMessage(message: unknown, options?: ExplorerWindowPostMessageOptions): void;
  function postMessage(message: unknown, targetOrigin: string, transfer: ExplorerTransferable[]): void;
  function structuredClone<T>(value: T, options: { transfer: ExplorerTransferable[] }): T;
  interface Window {
    postMessage(message: unknown, options: ExplorerWindowPostMessageOptions): void;
    postMessage(message: unknown, targetOrigin: string, transfer: ExplorerTransferable[]): void;
  }
  interface MessagePort {
    postMessage(message: unknown, transfer: ExplorerTransferable[]): void;
    postMessage(message: unknown, options: { transfer: ExplorerTransferable[] }): void;
  }
  interface HTMLElementTagNameMap {
    explorer: HTMLExplorerElement;
  }
}

export type ExplorerDownload = {
  id: number;
  url: string;
  filename: string;
  path: string;
  mimeType: string;
  state: "in-progress" | "complete" | "cancelled" | "interrupted";
  receivedBytes: number;
  totalBytes: number;
  error: number;
  navigation: boolean;
};

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

export interface ExplorerOpenDetail {
  readonly url: string;
  readonly disposition: string;
  readonly userGesture: boolean;
  readonly from: string;
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

export type ExplorerTransferState = "pending" | "attaching" | "transferred" | "closed";

/** Pending resources are owned by the source document, independently of GC. */
export interface ExplorerTransfer extends EventTarget {
  readonly token: string;
  readonly state: ExplorerTransferState;
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
  open: CustomEvent<ExplorerOpenDetail>;
  popup: CustomEvent<ExplorerOpenDetail>;
  newwindow: CustomEvent<ExplorerOpenDetail>;
  navigation: CustomEvent<ExplorerOpenDetail>;
  dialog: CustomEvent<ExplorerDialogController>;
  message: CustomEvent<unknown>;
}

export interface HTMLExplorerElement extends HTMLElement {
  src: string;
  /** Empty selects the parent's partition; `shared` is not an element value. */
  partition: string;
  navigation: ExplorerNavigationPolicy;
  dialogs: ExplorerDialogPolicy;
  contextMenuEnabled: boolean;
  deferred: boolean;
  detach(): Promise<ExplorerTransfer>;
  attach(token: string): Promise<void>;
  ondetached: ((event: Event) => void) | null;
  onattached: ((event: Event) => void) | null;
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
  /** The caller must validate the JSON-representable result at runtime. */
  executeJavaScript(
    source: string,
    options?: Readonly<{
      world?: ExplorerJavaScriptWorld;
      expectedDocumentId?: string;
    }>,
  ): Promise<unknown>;
  postMessage(data: unknown): Promise<boolean>;
  addEventListener<K extends keyof ExplorerEventMap>(
    type: K,
    listener: (this: HTMLExplorerElement, event: ExplorerEventMap[K]) => void,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener<K extends keyof ExplorerEventMap>(
    type: K,
    listener: (this: HTMLExplorerElement, event: ExplorerEventMap[K]) => void,
    options?: boolean | EventListenerOptions,
  ): void;
}

declare global {
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

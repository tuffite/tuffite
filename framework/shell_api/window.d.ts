/** Opaque browser-process identifier. It is intentionally not a native handle. */
export type WindowId = bigint;

export interface WindowBounds {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export interface WindowInfo {
  readonly id: WindowId;
  readonly label: string;
  readonly title: string;
  readonly url: string;
  readonly bounds: WindowBounds;
  readonly focused: boolean;
  readonly minimized: boolean;
  readonly maximized: boolean;
  readonly fullscreen: boolean;
}

export interface WindowOptions {
  readonly label?: string;
  readonly url?: string;
  readonly title?: string;
  readonly bounds?: WindowBounds;
  readonly focused?: boolean;
}

export declare namespace tuffite {
  export namespace window {
    export function current(): Promise<WindowInfo>;
    export function list(): Promise<WindowInfo[]>;
    export function create(options?: WindowOptions): Promise<WindowInfo>;
    export function focus(id?: WindowId): Promise<WindowInfo>;
    export function minimize(id?: WindowId): Promise<WindowInfo>;
    export function maximize(id?: WindowId): Promise<WindowInfo>;
    export function restore(id?: WindowId): Promise<WindowInfo>;
    export function setFullscreen(fullscreen: boolean, id?: WindowId): Promise<WindowInfo>;
    export function close(id?: WindowId): Promise<void>;
    export function setTitle(title: string, id?: WindowId): Promise<WindowInfo>;
    export function getBounds(id?: WindowId): Promise<WindowBounds>;
    export function setBounds(bounds: WindowBounds, id?: WindowId): Promise<WindowInfo>;
  }
}

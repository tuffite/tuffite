export type JsonValue = null | boolean | number | string | JsonValue[] | JsonObject;
export interface JsonObject { [key: string]: JsonValue }
export interface IsolatedRequest {
  readonly method: string;
  readonly params: JsonValue;
  readonly signal: AbortSignal;
}
export interface IsolatedMountOptions {
  tag: string;
  properties?: JsonObject;
  onRequest?(request: IsolatedRequest): JsonValue | void | Promise<JsonValue | void>;
}
/** Main-world manager. Packages and origin grants are registered by the application. */
export interface IsolatedHost {
  createWorld(options: { name: string }): Promise<IsolatedWorldHandle>;
}
export interface IsolatedWorldHandle extends EventTarget {
  readonly name: string;
  mount(container: HTMLElement, options: IsolatedMountOptions): Promise<IsolatedViewHandle>;
  dispose(): Promise<void>;
}
export interface IsolatedViewHandle {
  /** Assigns a top-level property patch inside the owning world, before paint. */
  update(properties: JsonObject): Promise<void>;
  dispose(): Promise<void>;
}
/** Injected into a DOM isolated world; shares the document and renderer thread. */
export interface IsolatedWorld {
  readonly name: string;
  readonly customElements: CustomElementRegistry;
  readonly signal: AbortSignal;
  context(element: HTMLElement): IsolatedViewContext;
}
export interface IsolatedViewContext {
  readonly signal: AbortSignal;
  request(method: string, params?: JsonValue, options?: { signal?: AbortSignal }): Promise<JsonValue>;
}
export declare const isolated: IsolatedHost | IsolatedWorld | undefined;

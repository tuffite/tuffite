export declare namespace tuffite {
  export namespace storage {
    export function get(key: string): Promise<Uint8Array | undefined>;
    export function set(key: string, value: Uint8Array): Promise<void>;
    export function remove(key: string): Promise<void>;
    export function clear(): Promise<void>;
  }
}

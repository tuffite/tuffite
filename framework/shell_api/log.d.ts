export declare namespace tuffite {
  export namespace log {
    export function trace(message: string): Promise<void>;
    export function debug(message: string): Promise<void>;
    export function info(message: string): Promise<void>;
    export function warn(message: string): Promise<void>;
    export function error(message: string): Promise<void>;
  }
}

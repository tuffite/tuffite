// Generated from ../../framework/shell_api/utility.d.ts. Do not edit.

export declare namespace tuffite {
  export namespace utility {
    export function invoke(
      name: string,
      requestId: bigint,
      payload?: Uint8Array,
    ): Promise<Uint8Array>;
    export function cancel(name: string, requestId: bigint): Promise<void>;
  }
}

// Generated from ../../framework/shell_api/network.d.ts. Do not edit.

export interface NetworkResponse {
  readonly status: number;
  readonly body: Uint8Array;
}

export declare namespace tuffite {
  export namespace net {
    export function request(url: string): Promise<NetworkResponse>;
  }
}

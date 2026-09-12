// Generated from ../../framework/shell_api/path.d.ts. Do not edit.

import type { BaseDirectory } from './common';

export interface PathDirectories {
  readonly resource: string;
  readonly data: string;
  readonly cache: string;
  readonly temp: string;
}

export declare namespace tuffite {
  export namespace path {
    export function directories(): Promise<PathDirectories>;
    export function resolve(base: BaseDirectory, relative?: string): Promise<string>;
  }
}

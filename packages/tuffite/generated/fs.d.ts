// Generated from ../../framework/shell_api/fs.d.ts. Do not edit.

import type { BaseDirectory } from './common';

export declare namespace tuffite {
  export namespace fs {
    export function readFile(base: BaseDirectory, path: string): Promise<Uint8Array>;
    export function writeFile(
      base: BaseDirectory,
      path: string,
      contents: Uint8Array,
    ): Promise<void>;
    export function exists(base: BaseDirectory, path: string): Promise<boolean>;
    export function createDir(base: BaseDirectory, path: string): Promise<void>;
    export function removeFile(base: BaseDirectory, path: string): Promise<void>;
  }
}

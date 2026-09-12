// Generated from ../../framework/shell_api/dialog.d.ts. Do not edit.

export interface OpenFileDialogOptions {
  readonly title?: string;
  readonly defaultPath?: string;
  /** Extension groups without leading dots, for example [["html", "htm"], ["md"]]. */
  readonly extensions?: string[][];
}

export interface OpenDirectoryDialogOptions {
  readonly title?: string;
}

export declare namespace tuffite {
  export namespace dialog {
    export function openFile(options?: OpenFileDialogOptions): Promise<string | undefined>;
    export function openDirectory(options?: OpenDirectoryDialogOptions): Promise<string | undefined>;
  }
}

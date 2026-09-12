export interface TuffiteViteOptions {
  manifest?: string;
  root?: string;
  static?: Readonly<Record<string, string>>;
  shellApi?: false | Readonly<{ url: string; file: string }>;
}

export function tuffite(options?: TuffiteViteOptions): import("vite").Plugin;
export function findManifest(start: string): string | undefined;
export function contentTypeForPath(path: string): string | undefined;
export function decodeRequestPath(requestUrl: string | undefined): string | undefined;
export function resolveConfinedFile(root: string, path: string): Promise<string | undefined>;

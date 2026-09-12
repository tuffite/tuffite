// Generated from ../../framework/shell_api/common.d.ts, ../../framework/shell_api/dialog.d.ts, ../../framework/shell_api/path.d.ts, ../../framework/shell_api/remoting.d.ts, ../../framework/shell_api/fs.d.ts, ../../framework/shell_api/storage.d.ts, ../../framework/shell_api/log.d.ts, ../../framework/shell_api/network.d.ts, ../../framework/shell_api/utility.d.ts, ../../framework/shell_api/window.d.ts. Do not edit.

declare global {
  interface Window {
    readonly tuffite: typeof import('./dialog').tuffite & typeof import('./path').tuffite & typeof import('./remoting').tuffite & typeof import('./fs').tuffite & typeof import('./storage').tuffite & typeof import('./log').tuffite & typeof import('./network').tuffite & typeof import('./utility').tuffite & typeof import('./window').tuffite & import('../runtime').NativeBridge;
  }
}

export {};

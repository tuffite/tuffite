/** Business ShellAPI declarations. Tuffite generates Rust types, dispatch, codecs,
 * the renderer Proxy, and global declarations from this file. */
export declare namespace playground {
  export namespace commands {
    export function doSomething(message: string): Promise<string>;
    export function checkUtilitySession(): Promise<string>;
    export function checkJavascriptExtension(): Promise<string>;
  }
}

// Generated from shell_api.d.ts. Do not edit.
import '/tuffite-shell-api.js';
import { createTypedCommandApi } from '/tuffite-runtime.js';

const api = createTypedCommandApi([
  [
    "commands.doSomething",
    "0x6a89a07098fef70b",
    "0xc8047a53ea8554b1",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "string"
    }
  ],
  [
    "commands.checkUtilitySession",
    "0x4b6d1542f927fbba",
    "0x7b7d2edc9cde71da",
    [],
    {
      "kind": "string"
    }
  ],
  [
    "commands.checkJavascriptExtension",
    "0x4595d71ce868de6e",
    "0x7b7d2edc9cde71da",
    [],
    {
      "kind": "string"
    }
  ]
], [
], "playground");
Object.defineProperty(window, "playground", { configurable: false, enumerable: true, writable: false, value: api });

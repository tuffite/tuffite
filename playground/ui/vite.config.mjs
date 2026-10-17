import { isAbsolute, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { cp, mkdir } from "node:fs/promises";
import { defineConfig } from "vite";
import { tuffite } from "../../packages/tuffite/vite.js";

const outDir = process.env.TUFFITE_PLAYGROUND_UI_OUT_DIR || "dist";
const uiRoot = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  base: "./",
  publicDir: false,
  plugins: [
    tuffite({
      root: fileURLToPath(new URL("..", import.meta.url)),
      shellApi: { url: "/shell_api.js", file: "generated/index.js" },
      static: { "/playground-public/": "ui/public" },
    }),
    {
      name: "playground-public-assets",
      apply: "build",
      async closeBundle() {
        const output = resolve(
            isAbsolute(outDir) ? outDir : resolve(uiRoot, outDir),
            "playground-public");
        await mkdir(output, { recursive: true });
        await cp(fileURLToPath(new URL("./public", import.meta.url)), output, { recursive: true });
      },
    },
  ],
  server: { host: "127.0.0.1", port: 5189, strictPort: true },
  build: {
    outDir,
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL("./index.html", import.meta.url)),
        capabilities: fileURLToPath(new URL("./capabilities.html", import.meta.url)),
        compatibility: fileURLToPath(new URL("./compatibility.html", import.meta.url)),
        compatibilitySample: fileURLToPath(new URL("./compatibility-sample.html", import.meta.url)),
        events: fileURLToPath(new URL("./events.html", import.meta.url)),
        eventFixture: fileURLToPath(new URL("./event-fixture.html", import.meta.url)),
        eventPopup: fileURLToPath(new URL("./event-popup.html", import.meta.url)),
        tabs: fileURLToPath(new URL("./tabs.html", import.meta.url)),
        tabSample: fileURLToPath(new URL("./tab-sample.html", import.meta.url)),
        capabilitySample: fileURLToPath(new URL("./capability-sample.html", import.meta.url)),
        transfer: fileURLToPath(new URL("./transfer.html", import.meta.url)),
        transferSample: fileURLToPath(new URL("./transfer-sample.html", import.meta.url)),
        explorer: fileURLToPath(new URL("./explorer.html", import.meta.url)),
        nested: fileURLToPath(new URL("./nested.html", import.meta.url)),
        historyA: fileURLToPath(new URL("./history-a.html", import.meta.url)),
        historyB: fileURLToPath(new URL("./history-b.html", import.meta.url)),
        window: fileURLToPath(new URL("./window.html", import.meta.url)),
      },
    },
  },
});

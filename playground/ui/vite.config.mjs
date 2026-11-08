import { isAbsolute, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cp, mkdir } from 'node:fs/promises';
import { defineConfig } from 'vite';
import { tuffite } from '../../packages/tuffite/vite.js';

const outDir = process.env.TUFFITE_PLAYGROUND_UI_OUT_DIR || 'dist';
const uiRoot = fileURLToPath(new URL('.', import.meta.url));

export default defineConfig({
  base: './',
  publicDir: false,
  plugins: [
    tuffite({
      root: fileURLToPath(new URL('..', import.meta.url)),
      shellApi: { url: '/shell_api.js', file: 'generated/index.js' },
      static: { '/playground-public/': 'ui/public' },
    }),
    {
      name: 'playground-public-assets',
      apply: 'build',
      async closeBundle() {
        const output = resolve(
          isAbsolute(outDir) ? outDir : resolve(uiRoot, outDir),
          'playground-public',
        );
        await mkdir(output, { recursive: true });
        await cp(fileURLToPath(new URL('./public', import.meta.url)), output, { recursive: true });
      },
    },
  ],
  server: { host: '127.0.0.1', port: 5189, strictPort: true },
  build: {
    outDir,
    emptyOutDir: true,
    rolldownOptions: {
      input: [
        'index.html',
        'profiles.html',
        'framework.html',
        'isolated.html',
        'capabilities.html',
        'compatibility.html',
        'events.html',
        'behavior.html',
        'tabs.html',
        'transfer.html',
        'explorer.html',
        'nested.html',
        'window.html',
        'fixtures/behavior.html',
        'fixtures/events.html',
        'fixtures/popup.html',
        'fixtures/compatibility.html',
        'fixtures/tab.html',
        'fixtures/capabilities.html',
        'fixtures/transfer.html',
        'fixtures/mojom.html',
        'fixtures/configuration.html',
        'fixtures/history.html',
      ].map((entry) => fileURLToPath(new URL(entry, import.meta.url))),
    },
  },
});

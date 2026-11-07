import assert from 'node:assert/strict';
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { PassThrough } from 'node:stream';
import { resourceMounts, resourceExcluded, tuffite } from '../packages/tuffite/vite.js';
const root = await mkdtemp(join(tmpdir(), 'tuffite-resource-mounts-'));
try {
  await mkdir(join(root, 'ui/dist'), { recursive: true });
  await mkdir(join(root, 'extra'));
  await writeFile(join(root, 'ui/dist/index.html'), 'index');
  await writeFile(join(root, 'ui/dist/index.js.map'), 'excluded');
  await writeFile(join(root, 'extra/logo.png'), 'image');
  const resources = { ui: ['ui/dist', 'extra', '!**/*.map', '!**/ignore'], '.': ['extra/logo.png'] };
  const mounts = resourceMounts(resources, root);
  assert.deepEqual(mounts.map(m => m.prefix), ['/ui/', '/ui/', '/logo.png']);
  assert.equal(resourceExcluded(['**/ignore'], ['nested/ignore/child.txt']), true);
  assert.equal(resourceExcluded(['**/ignore'], ['.hidden/ignore/child.txt']), true);
  assert.equal(resourceExcluded(['**/*.map'], ['index.js.map']), true);
  assert.equal(resourceExcluded(['**/*.map'], ['nested/index.js']), false);
  assert.throws(() => resourceMounts(['ui'], root), /alias object/);
  assert.throws(() => resourceMounts({ '../escape': ['ui'] }, root), /unsafe/);
  const manifest = join(root, 'tuffite.json');
  await writeFile(manifest, JSON.stringify({ bundle: { resources } }));
  let middleware;
  tuffite({ manifest, shellApi: false }).configureServer({ middlewares: { use(handler) { middleware = handler; } } });
  async function request(url) {
    return new Promise((resolve, reject) => {
      const response = new PassThrough();
      response.setHeader = () => {};
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('end', () => resolve({ body: Buffer.concat(chunks).toString() }));
      middleware({ url }, response, error => error ? reject(error) : resolve({ next: true })).catch(reject);
    });
  }
  assert.deepEqual(await request('/ui/index.html'), { body: 'index' });
  assert.deepEqual(await request('/ui/logo.png'), { body: 'image' });
  assert.deepEqual(await request('/logo.png'), { body: 'image' });
  assert.deepEqual(await request('/ui/index.js.map'), { next: true });
  console.log('Resource alias development mounts passed');
} finally { await rm(root, { recursive: true, force: true }); }

import { fixtureId } from './generated.mjs';
import { cycleA } from './cycle-a.mjs';
import { remoteFixture } from 'https://fixtures.tuffite.invalid/sdk/remote.mjs';
import { localFixture } from 'file:///tuffite-fixtures/sdk/local.mjs';
export function describeFixture() {
  let evalBlocked = false;
  try { eval('1'); } catch { evalBlocked = true; }
  return {
    id: fixtureId,
    staticImport: true,
    cycleImport: cycleA(2) === 'cycle',
    standardUrls: remoteFixture && localFixture,
    evalBlocked,
    ambientAuthority: {
      process: typeof process,
      require: typeof require,
      fetch: typeof fetch
    }
  };
}

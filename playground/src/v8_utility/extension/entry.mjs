import { describeFixture } from '../sdk/runtime.mjs';
import { fixtureId, sharedState } from '../sdk/generated.mjs';
export default Object.freeze({
  describe: () => ({ ...describeFixture(), sharedState, sharedImport: fixtureId === 'io.tuffite.playground' }),
  spin: () => { while (true) {} }
});

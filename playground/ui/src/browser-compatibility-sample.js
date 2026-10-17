import { runBrowserCompatibilityProbe } from './browser-compatibility-probe.js';
document.body.dataset.compatibilityReport = JSON.stringify(await runBrowserCompatibilityProbe());

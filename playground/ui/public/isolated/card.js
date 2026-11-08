import sheet from './card.css' with { type: 'css' };
import data from './weather.json' with { type: 'json' };
import { label, moduleUrl } from './details.js';

const icon = new URL('./weather.svg', import.meta.url).href;
const lazy = await import('./lazy.js');
let escapeDenied = false;
try { await import('../outside.js'); } catch { escapeDenied = true; }
let networkDenied = false;
try { await import('https://example.invalid/not-allowed.js'); } catch { networkDenied = true; }
await new Promise(resolve => setTimeout(resolve, 5));

(() => {
  const isolated = tuffite.isolated;
  globalThis.__worldSecret = isolated.name;
  class Detail extends HTMLElement {
    connectedCallback() { this.textContent = 'nested component'; }
  }
  class Card extends HTMLElement {
    #root;
    #data;
    constructor() {
      super();
      this.#root = this.attachShadow({mode: 'open', customElementRegistry: isolated.customElements});
      this.#root.adoptedStyleSheets = [sheet];
    }
    set data(value) { this.#data = value; this.render(); }
    set streaming(value) { this.toggleAttribute('data-streaming', value); }
    set probe(value) {
      if (value === 'pending') {
        void this.context.request('wait', {}).catch(error => {
          this.dataset.cancelled = String(error.name === 'AbortError');
        });
      }
      if (value === 'cancel') {
        const controller = new AbortController();
        void this.context.request('wait', {}, {signal: controller.signal}).catch(error => {
          this.dataset.cancelled = String(error.name === 'AbortError');
        });
        setTimeout(() => controller.abort(), 0);
      }
    }
    static observedAttributes = ['data-probe'];
    attributeChangedCallback() {
      this.dataset.callbackWorld = isolated.name;
      this.dataset.mainSecret = String(globalThis.__hostSecret);
    }
    connectedCallback() {
      this.context = isolated.context(this);
      this.dataset.world = isolated.name;
      this.dataset.modules = JSON.stringify({label, moduleUrl, lazy: lazy.value, city: data.city, escapeDenied, networkDenied});
      try { isolated.context(document.body); this.dataset.foreignDenied = 'false'; }
      catch { this.dataset.foreignDenied = 'true'; }
      this.dataset.registryLocal = String(this.customElementRegistry === isolated.customElements);
      this.dataset.mainRegistry = String(window.customElements);
      this.dataset.shellApi = String(typeof globalThis.Mojo?.bindInterface);
      this.context.signal.addEventListener('abort', () => { this.dataset.aborted = 'true'; });
      this.render();
    }
    render() {
      if (!this.#data) return;
      this.#root.innerHTML = '<h2></h2><img alt="Weather"><world-detail></world-detail><p></p><button>Ask host</button>';

      this.#root.querySelector('img').src = icon;
      this.#root.querySelector('h2').textContent = this.#data.title;
      this.#root.querySelector('p').textContent = `${isolated.name}: ${this.#data.value}`;
      this.#root.querySelector('button').addEventListener('click', async () => {
        try {
          const result = await this.context.request('echo', {value: this.#data.value});
          this.dataset.reply = JSON.stringify(result);
          this.#root.querySelector('p').textContent = `Host replied: ${result.value}`;
        } catch (error) { this.dataset.error = error.message; }
      });
    }
  }
  isolated.customElements.define('world-detail', Detail);
  isolated.customElements.define('weather-card', Card);
})();

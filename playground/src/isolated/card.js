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
      try { isolated.context(document.body); this.dataset.foreignDenied = 'false'; }
      catch { this.dataset.foreignDenied = 'true'; }
      this.dataset.registryLocal = String(this.customElementRegistry === isolated.customElements);
      this.dataset.mainRegistry = String(window.customElements);
      this.dataset.shellApi = String(typeof tuffite.invoke);
      this.context.signal.addEventListener('abort', () => { this.dataset.aborted = 'true'; });
      this.render();
    }
    render() {
      if (!this.#data) return;
      this.#root.innerHTML = '<style>:host{display:block;padding:16px;border:1px solid #888;border-radius:12px}button{font:inherit}h2{margin-top:0}</style><h2></h2><world-detail></world-detail><p></p><button>Ask host</button>';
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

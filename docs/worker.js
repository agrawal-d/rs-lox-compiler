import init, { run } from './wasm.js';

onmessage = function(e) {
    run(e.data);
    this.postMessage(null);
}

await init();
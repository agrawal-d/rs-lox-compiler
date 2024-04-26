import init, { run } from './wasm.js';

onmessage = function(e) {
    try{
        run(e.data);
    }
    finally{
        this.postMessage(null);
    }
}

await init();
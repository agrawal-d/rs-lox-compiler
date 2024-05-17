import init, { run } from './wasm.js';

self.userInput = null;

onmessage = function (e) {
    try {
        let message = e.data;
        if (message.type === "run") {
            run(message.code);
        }
        else if (message.type === "input-response") {
            self.userInput = message.data;
        }
        else {
            console.error("Invalid message", e);
        }
    }
    finally {
        this.postMessage({
            type: "run-end"
        });
    }
}

await init();
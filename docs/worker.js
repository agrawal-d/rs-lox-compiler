import init, { run } from './wasm.js';

self.userInput = null;

onmessage = function (e) {
    try {
        let message = e.data;
        if (message.type === "run") {
            run(message.code).then(() => {
                this.postMessage({
                    type: "run-end"
                });
            });
        }
        else if (message.type === "input-response") {
            self.userInput = message.data;
        }
        else {
            console.error("Invalid message", e);
        }
    }
    catch (e) {
        console.error(e);

        this.postMessage({
            type: "output",
            data: e.toString()
        });

        this.postMessage({
            type: "run-end"
        });
    }
}

await init();
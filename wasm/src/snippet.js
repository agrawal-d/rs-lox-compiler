console.log("Snippets init inside worker");

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export function print(output) {
    postMessage({
        type: "output",
        data: output
    });
}

export function println(output) {
    postMessage({
        type: "output",
        data: output + "\n"
    });
}

export async function readAsync(text) {
    postMessage({
        type: "input-request",
        prompt: text
    });

    while (self.userInput === null) {
        console.log("Waiting for user to respond...");
        await sleep(100);
    }

    let input = self.userInput;
    self.userInput = null;
    return input;
}

export function end() {
    postMessage({
        type: "run-end"
    });
}
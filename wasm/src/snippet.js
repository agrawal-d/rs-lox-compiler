console.log("Snippets init inside worker");

export function print(output) {
    postMessage(output);
}

export function println(output) {
    postMessage(output + "\n");
}

export function read(text) {
    return "Input not implemented for web";
}
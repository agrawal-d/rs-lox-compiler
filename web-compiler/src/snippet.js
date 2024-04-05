const outputTextarea = document.getElementById('outputTextarea');

export function print(output) {
    outputTextarea.value += output;
}

export function println(output) {
    outputTextarea.value += output;
    outputTextarea.value += '\n';
}

export function resetOutput() {
    outputTextarea.value = "";
}
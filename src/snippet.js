const outputTextarea = document.getElementById('outputTextarea');

export function appendOutput(output) {
    outputTextarea.value += output;
}

export function resetOutput() {
    outputTextarea.value = "";
}
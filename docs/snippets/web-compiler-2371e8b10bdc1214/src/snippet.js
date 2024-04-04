const outputTextarea = document.getElementById('outputTextarea');

export function print(output) {
    outputTextarea.value += output;
}

export function resetOutput() {
    outputTextarea.value = "";
}
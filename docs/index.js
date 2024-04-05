import init, { run } from './wasm.js';

const inputTextarea = document.getElementById('inputTextarea');
const outputTextarea = document.getElementById('outputTextarea');
const runButton = document.getElementById('runButton');
const resetButton = document.getElementById('resetButton');

runButton.addEventListener('click', async () => {
    const input = inputTextarea.value;
    run(input);
    outputTextarea.focus();
});

resetButton.addEventListener('click', () => {
    inputTextarea.value = '';
    outputTextarea.value = '';
    inputTextarea.focus();
    console.clear();
});

await init();
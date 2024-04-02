import init, { run_code } from './web_compiler.js';

const inputTextarea = document.getElementById('inputTextarea');
const outputTextarea = document.getElementById('outputTextarea');
const runButton = document.getElementById('runButton');
const resetButton = document.getElementById('resetButton');

runButton.addEventListener('click', async () => {
    const input = inputTextarea.value;
    run_code(input);
    outputTextarea.focus();
});

resetButton.addEventListener('click', () => {
    inputTextarea.value = '';
    outputTextarea.value = '';
    inputTextarea.focus();
    console.clear();
});

await init();
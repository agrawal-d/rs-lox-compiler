import init, { run_code } from './pkg/web_compiler.js';

const inputTextarea = document.getElementById('inputTextarea');
const outputTextarea = document.getElementById('outputTextarea');
const runButton = document.getElementById('runButton');
const resetButton = document.getElementById('resetButton');

runButton.addEventListener('click', async () => {
    const input = inputTextarea.value;
    run_code(input);
});


await init();
const myWorker = new Worker('worker.js', { type: 'module' });
///// Monaco

console.log("Setting up Monaco Editor");
require.config({ paths: { 'vs': 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.48.0/min/vs' } });
require(["vs/editor/editor.main"], function () {
    window.editor = monaco.editor.create(document.getElementById('editor'), {
        value: 'var counter = 10;\n\nwhile (counter > 0)\n{\n    print(counter);\n    counter = counter - 1;\n}\n\nprint("Liftoff!");',
        language: 'csharp',
        scrollBeyondLastLine: false,
        minimap: { enabled: false },
        automaticLayout: true,
    });
});


///// Compiler

const outputTextarea = document.getElementById('outputTextarea');
const runButton = document.getElementById('runButton');
const resetButton = document.getElementById('resetButton');
const statsP = document.getElementById('stats');

myWorker.onmessage = function (e) {
    outputTextarea.value += e.data;
};

runButton.addEventListener('click', async () => {
    outputTextarea.value = '';
    runButton.innerText = 'Running...';
    runButton.disabled = true;
    const input = window.editor.getValue();
    const starts = performance.now();
    myWorker.postMessage(input);
    const endts = performance.now();
    const msTaken = endts - starts;
    outputTextarea.focus();
    statsP.innerText = `Execution time ${msTaken.toFixed(2)} ms`;
    runButton.disabled = false;
    runButton.innerText = 'Run';
});

resetButton.addEventListener('click', () => {
    statsP.innerText = 'Ready';
    window.editor.setValue('');
    outputTextarea.value = '';
    editor.focus();
    console.clear();
});

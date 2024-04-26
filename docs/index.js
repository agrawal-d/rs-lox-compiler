const myWorker = new Worker('worker.js', { type: 'module' });
///// Monaco

console.log("Setting up Monaco Editor");
require.config({ paths: { 'vs': 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.48.0/min/vs' } });
require(["vs/editor/editor.main"], function () {
    window.editor = monaco.editor.create(document.getElementById('editor'), {
        value: 'for(var i = 0; i < 10000000; i = i + 1)\n{\n    if(i % 1000000 == 0)\n    {\n        print("Crunched numbers: " + i);\n    }\n}\n\nprint("Crunching numbers complete!");',
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
var starts = 0;

myWorker.onmessage = function (e) {
    if (e.data == null) {
        const endts = performance.now();
        const msTaken = endts - starts;
        statsP.innerText = `Execution time ${msTaken.toFixed(2)} ms`;

        runButton.disabled = false;
        resetButton.disabled = false;
        runButton.innerText = 'Run';
    } else {
        outputTextarea.value += e.data;
    }

};

runButton.addEventListener('click', async () => {
    runButton.disabled = true;
    resetButton.disabled = true;
    runButton.innerText = 'Running...';
    statsP.innerHTML = "<div class='loader'></div>";
    outputTextarea.value = '';
    const input = window.editor.getValue();
    starts = performance.now();
    myWorker.postMessage(input);
    outputTextarea.focus();
});

resetButton.addEventListener('click', () => {
    statsP.innerText = 'Ready';
    window.editor.setValue('');
    outputTextarea.value = '';
    editor.focus();
    console.clear();
});

outputTextarea.value = '';
const myWorker = new Worker('worker.js', { type: 'module' });

///// Monaco
console.log("Setting up Monaco Editor");

require.config({ paths: { 'vs': 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.48.0/min/vs' } });
require(["vs/editor/editor.main"], function () {
    window.editor = monaco.editor.create(document.getElementById('editor'), {
        value: 'print("Please wait, samples are loading...")',
        language: 'csharp',
        scrollBeyondLastLine: false,
        minimap: { enabled: false },
        automaticLayout: true,
    });

    loadSamples();
});


///// Compiler

const outputTextarea = document.getElementById('outputTextarea');
const runButton = document.getElementById('runButton');
const resetButton = document.getElementById('resetButton');
const statsP = document.getElementById('stats');
var starts = 0;

myWorker.onmessage = async function (e) {
    let message = e.data
    if (message.type == "output") {
        outputTextarea.value += message.data;
    } else if (message.type == "run-end") {
        const endts = performance.now();
        const msTaken = endts - starts;
        statsP.innerText = `Execution time ${msTaken.toFixed(2)} ms`;

        runButton.disabled = false;
        resetButton.disabled = false;
        runButton.innerText = 'Run';
    } else if (message.type == "input-request") {
        console.log("Input requested");
        myWorker.postMessage({
            type: 'input-response',
            data: await customPrompt(message.prompt)
        });
    }

    else {
        console.error("Invalid message", message);
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
    myWorker.postMessage({
        type: 'run',
        code: input
    });
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


///// Sample picker

const samplePicker = document.getElementById('sample-select');

function loadSamples() {
    var samples = [
        {
            name: "Fibonacci (Iterative)",
            code: "sample_programs/fib_iterative.lox",
        },
        {
            name: "Fibonacci (Recursive)",
            code: "sample_programs/fib_recursive.lox",
        },
        {
            name: "Guess Game",
            code: "sample_programs/guess_game.lox",
        },
        {
            name: "Story Generator",
            code: "sample_programs/interactive.lox",
        }, {
            name: "Merge Sort",
            code: "sample_programs/merge_sort.lox",
        }, {
            name: "Trace Back",
            code: "sample_programs/traceback.lox",
        }
    ]

    // For each sample, fetch the code and add it to the dropdown, and also update samples with the real code

    var promises = [];
    for (let i = 0; i < samples.length; i++) {
        var option = document.createElement("option");
        option.text = samples[i].name;
        samplePicker.add(option);
        promises.push(fetch(samples[i].code)
            .then(response => response.text())
            .then(data => {
                samples[i].code = data;
            }));
    }

    // On select choose, update value of editor
    samplePicker.addEventListener('change', () => {
        const selected = samplePicker.selectedIndex;
        window.editor.setValue(samples[selected].code);
        editor.focus();
    });

    Promise.all(promises).then(() => {
        console.log("Samples loaded");
        window.editor.setValue(samples[0].code);
        editor.focus();
    });
}


///// Prompt

window.customPrompt = (promptMessage) => {
    return new Promise((resolve) => {
        // Create the overlay
        const overlay = document.createElement('div');
        overlay.classList.add('custom-prompt-overlay');

        // Create the prompt container
        const promptContainer = document.createElement('div');
        promptContainer.classList.add('custom-prompt-container');

        // Create the prompt message
        const promptText = document.createElement('p');
        promptText.textContent = promptMessage;

        // Create the input field
        const inputField = document.createElement('input');
        inputField.type = 'text';

        // Create the buttons container
        const buttonsContainer = document.createElement('div');
        buttonsContainer.classList.add('custom-prompt-buttons');


        function handleInput() {
            resolve(inputField.value);
            document.body.removeChild(overlay);
        }

        // Create the OK button
        const okButton = document.createElement('button');
        okButton.textContent = 'OK';
        okButton.classList.add('ok-button');
        okButton.classList.add('green');
        okButton.addEventListener('click', handleInput);

        inputField.addEventListener('keyup', function (event) {
            if (event.key === 'Enter') {
                handleInput();
            }
        });

        document.addEventListener('keydown', function (event) {
            if (event.key === 'Escape') {
                resolve('');
                document.body.removeChild(overlay);
            }
        });

        // Create the Cancel button
        const cancelButton = document.createElement('button');
        cancelButton.textContent = 'Cancel';
        cancelButton.classList.add('cancel-button');
        cancelButton.addEventListener('click', () => {
            resolve('');
            document.body.removeChild(overlay);
        });

        // Append elements
        buttonsContainer.appendChild(okButton);
        buttonsContainer.appendChild(cancelButton);
        promptContainer.appendChild(promptText);
        promptContainer.appendChild(inputField);
        promptContainer.appendChild(buttonsContainer);
        overlay.appendChild(promptContainer);
        document.body.appendChild(overlay);

        // Focus the input field
        inputField.focus();
    });
}
const myWorker = new Worker('worker.js', { type: 'module' });

///// Monaco
console.log("Setting up Monaco Editor");

require.config({ paths: { 'vs': 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.48.0/min/vs' } });
require(["vs/editor/editor.main"], function () {
    // Register Lox language
    monaco.languages.register({ id: 'lox' });

    // Define syntax rules
    monaco.languages.setMonarchTokensProvider('lox', {
        keywords: [
            'and', 'class', 'else', 'false', 'function', 'for', 'if', 'nil', 'or',
            'print', 'printf', 'return', 'super', 'this', 'true', 'var', 'while', 'import', 'as'
        ],
        builtins: [
            'clock', 'sleep', 'typeof', 'str', 'int', 'float', 'bool', 'stringat',
            'len', 'ceil', 'floor', 'abs', 'sort', 'indexof', 'rand', 'input',
        ],
        tokenizer: {
            root: [
                [/[a-zA-Z_]\w*/, {
                    cases: {
                        '@keywords': 'keyword',
                        '@builtins': 'predefined',
                        '@default': 'identifier'
                    }
                }],
                { include: '@whitespace' },
                [/[{}()\[\]]/, '@brackets'],
                [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
                [/\d+/, 'number'],
                [/"([^"\\]|\\.)*$/, 'string.invalid'],
                [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
            ],
            string: [
                [/[^\\"]+/, 'string'],
                [/\\./, 'string.escape.invalid'],
                [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }]
            ],
            whitespace: [
                [/[ \t\r\n]+/, 'white'],
                [/\/\*/, 'comment', '@comment'],
                [/\/\/.*$/, 'comment'],
            ],
            comment: [
                [/[^\/*]+/, 'comment'],
                [/\/\*/, 'comment', '@push'],
                [/\*\//, 'comment', '@pop'],
                [/[\/*]/, 'comment']
            ],
        }
    });

    // Define autocomplete
    monaco.languages.registerCompletionItemProvider('lox', {
        provideCompletionItems: (model, position) => {
            const word = model.getWordUntilPosition(position);
            const range = new monaco.Range(
                position.lineNumber,
                word.startColumn,
                position.lineNumber,
                word.endColumn
            );

            const suggestions = [
                ...[
                    'and', 'class', 'else', 'false', 'function', 'for', 'if', 'nil', 'or',
                    'print', 'return', 'super', 'this', 'true', 'var', 'while', 'import', 'as'
                ].map(k => ({
                    label: k,
                    kind: monaco.languages.CompletionItemKind.Keyword,
                    insertText: k,
                    range: range
                })),
                ...[
                    'clock', 'sleep', 'typeof', 'str', 'int', 'float', 'bool', 'stringat',
                    'len', 'ceil', 'floor', 'abs', 'sort', 'indexof', 'rand', 'input',
                ].map(b => ({
                    label: b,
                    kind: monaco.languages.CompletionItemKind.Function,
                    insertText: b + '($1)',
                    insertTextRules: 4, // CompletionItemInsertRule.InsertAsSnippet
                    range: range
                }))
            ];
            return { suggestions: suggestions };
        }
    });

    window.editor = monaco.editor.create(document.getElementById('editor'), {
        value: 'print("Please wait, samples are loading...")',
        language: 'lox',
        scrollBeyondLastLine: false,
        minimap: { enabled: false },
        automaticLayout: true,
        fontFamily: 'Consolas, "Ubuntu Mono", "Courier New", Courier, monospace',
        fontSize: 16,
    });

    loadSamples();
});


///// Compiler

const outputTextarea = document.getElementById('outputTextarea');
const runButton = document.getElementById('runButton');
const resetButton = document.getElementById('resetButton');
const statsP = document.getElementById('stats');
var starts = 0;

const runFn = () => {
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
}

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
    runFn();
});

document.addEventListener("keydown", (event) => {
    const isReloadShortcut =
        (event.ctrlKey || event.metaKey) &&
        event.key.toLowerCase() === "r";

    if (isReloadShortcut) {
        event.preventDefault();
        runFn();
    }
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
        },
        {
            name: "Write your own",
            code: "sample_programs/blank.lox",
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
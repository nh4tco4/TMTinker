# Turing Machine Tinker

Build Turing machine diagrams and watch them run on a tape. TMTinker is an editor and simulator for studying algorithms and teaching computer science, built with Rust and egui.

**[Try it in your browser](https://tmtinker.pages.dev/)** *UNSTABLE AND NOT FULLY IMPLEMENTED*

![Running a Turing machine: active nodes and tape changes](assets/readme/run_demo.gif)

## Features

- Draw machines with start, stop, move and write nodes. Connect them with transitions for the symbols in your alphabet.
- Reuse submachines and add notes to explain your diagrams.
- Run a machine, pause it or advance one step at a time. Adjust the step interval and follow the active node and tape head.
- Edit the tape as text or change individual cells and the head position.
- Save up to six tests per project with initial and expected tapes. Run them with a step limit and compare results relative to the head.
- Manage projects in a library, undo and redo edits, and import or export projects as JSON.
- Edit and run TU4 quadruples, or open a diagram as a separate TU4 project with its tape and tests.
- Run without delay in batches. TU4 supports programs of 400,000 commands and highlights the executing line for sources up to 2,500 lines.

## Screenshots

![TMTinker main menu and project library](assets/readme/menu.png)

![TMTinker edtior demonstation](assets/readme/editor.png)

## Your first machine

1. Open the app, choose **New project**, and enter a name and alphabet. You get the blank symbol `λ` as part of the alphabet.
2. Place nodes on the canvas. Choose **Link**, then click the source and target nodes. Select the transition to edit its symbols.
3. Choose **Validate** to check the diagram, then **Run diagram** to open the runner.
4. Enter a tape and choose **Apply tape**. Use parentheses to mark the head cell: `01(1)0`. For an empty tape, use `(λ)`.
5. Choose **Run** to start or **Step** to advance one step. Add initial and expected tapes to the test slots, then choose **Check all six** to run your saved tests.

Press **F1** or click **?** in the editor or runner for keyboard shortcuts and mouse controls. Use **Export project** to keep a JSON copy of your work.

## TU4 programs

Choose **TU4 Quadruples** when creating a project, or click **Open project in TU4** in the diagram editor. Conversion creates a separate project and preserves the original diagram. Submachine calls and repeated moves expand into quadruples, so execution can take more steps. Recursive submachine calls are prohibited in diagrams.

Write one command per line as `state,symbol,action,next_state`. For an empty tape, this program moves right, writes `1`, and halts:

```text
1,λ,>,2 // move right
2,λ,1,3 // write 1
3,1,~,0 // halt; ignore the fourth field
```

Use `<` for left, `>` for right, and `~` for halt. Other actions write a symbol from the project alphabet; `#` and `=` are ordinary symbols. Use `λ` or a literal space for blank. State numbers are unsigned 32-bit integers, and execution starts in the state of the first command. Writing the same symbol and staying in the same state continues the loop.

**Run** compiles the source and locks editing until you return with **Editor**. The runner shares the diagram's tape controls and six test slots. **Run without delay** executes batches while keeping Pause available. Sources above 2,500 lines use a paged editor and display the current state and line number during execution. Saving and JSON export include the source text, comments, prepared tape and tests, including drafts that do not compile yet.

## Run from source

You need Rust 1.88 or newer.

```bash
git clone https://github.com/nh4tco4/TMTinker.git
cd TMTinker
cargo run --release
```

Windows, trunk and macOS builds may be unstable.

To serve the web version on your machine, install Trunk and the WebAssembly target:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve --open
```

## Roadmap

The current priority is to stabilize the editor and runner. The target for these additions and improvements is the end of November 2026:

- Built-in machines: copy a word, move one word left and move one word right.
- Conversion from TU4 to diagrams.
- A Russian interface.
- PDF, SVG and PNG export.
- Improved Windows, macOS and web support.

## Contributing

Open an issue with steps to reproduce a bug or a description of the feature you need. For code changes, include what you changed and how you checked it in your pull request.

Run `./check.sh` for the repository checks. You need Trunk and the `wasm32-unknown-unknown` target for the web checks.

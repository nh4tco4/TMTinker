
# Turing Machine Tinker

An academic turing machine diagram editor for fundamental computer science related subjects.
Turing machine presented here is based on T4 implementation, meaning each action of machine has
starting state, symbol to detect, symbol to place, ending state. 

<img width="1280" height="825" alt="image" src="https://github.com/user-attachments/assets/7c63fbd9-bf51-4bc7-8eec-126677bc3cbe" />

You can try out the demo on cloudflare pages: https://tmtinker.pages.dev/
Or compile it natively:
```bash
cargo build --release
cargo run
```


**Still under development**
*Version 1.0 and manual are coming in the next few months.*

---


The interface includes a project library with autosave and JSON import/export, docked editor controls, undo/redo, and an indexed tape editor. Each project has six tests with initial and expected tapes. Use **Check all six** to compare the results, including head positions.

Keyboard shortcuts: Ctrl/⌘+Z for undo, Ctrl/⌘+Y or Ctrl/⌘+Shift+Z for redo, Ctrl/⌘+S for save, P to open the runner, V to validate, and Ctrl/⌘+N to create a submachine inside a project (or a project in the library). In the runner, P runs or pauses, Space performs one step, Ctrl/⌘+C resets, and Ctrl/⌘+D stops. Undo and redo are available in the editor. Buttons and the cheatsheet show the shortcuts.

Quadruple projects, documentation links and submachine export are marked **Soon**. See [UI architecture and verification](docs/ui-architecture.md) for the implementation and test commands.

Use the arrow keys to move selected nodes by one grid cell. Select a note to edit its text. Open **Cheatsheet** in the editor or runner, or press **F1**, for keyboard and mouse controls. Close it with **Escape**. Delete projects from the library after confirming the project name.

# thre 0.2.1

This release adds first-class text selection and a lightweight Markdown reading mode.

## Text selection and copying

- Press `Ctrl+A` to select the entire document.
- Hold `Shift` while using the arrow keys to extend a selection.
- Drag with the left mouse button to select text directly.
- Press `Ctrl+C` to copy the selected text through the terminal clipboard.
- Typing, pressing Enter, Backspace, or Delete replaces or removes the selection as expected.

## Markdown reading

Open a Markdown file with `--read_markdown` to apply terminal-friendly formatting for headings, block quotes, fenced blocks, bold text, emphasis, and inline code:

```sh
thre --read_markdown README.md
```

## Installation

Linux builds are available for x86-64 and ARM64. The installer asks which one to
download and uses x86-64 when you press Enter.

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/logancammish/thre/main/install.sh | sh
```

# thre 0.1.1

This release makes mouse-driven editing predictable and fixes the visible cursor being offset from the actual insertion point.

## Mouse improvements

- Click anywhere in the document to place the editing cursor there.
- Scroll with the mouse wheel without the cursor unexpectedly jumping to the top of the viewport.
- Use `Shift`+drag to select text with the terminal, then use the terminal's normal copy shortcut (usually `Ctrl+Shift+C`).
- Click file tabs to switch between open documents.

## Cursor fixes

- Corrected the one-cell mismatch between the line-number gutter and the document.
- Cursor placement now accounts for wide Unicode characters.
- Clicking wrapped lines maps to the correct document position.

## Installation

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/logancammish/thre/main/install.sh | sh
```

# thre 0.2.0

This release opens up `thre`'s appearance and highlighting system. You can now add your own themes and syntax definitions without rebuilding the editor, and shell scripts are highlighted out of the box.

## Custom themes

- Add `NAME.theme` files under `$XDG_CONFIG_HOME/thre/themes`, or `~/.config/thre/themes` when `XDG_CONFIG_HOME` is not set.
- Select custom themes with `--theme NAME` or cycle through them with `F2` and `Ctrl+T`.
- Discover both built-in and custom themes with `--list-themes`.
- Define only the colors you want to change; omitted values inherit from Midnight.

## Custom syntax highlighting

- Add `NAME.syntax` files under `$XDG_CONFIG_HOME/thre/syntaxes`.
- Associate definitions with one or more file extensions.
- Configure the display name, keywords, and line-comment marker using the documented compact format.
- Select a custom definition explicitly with `--language` using one of its configured extensions.

## Shell scripts

Shell syntax highlighting is now built in for `.sh`, `.bash`, `.zsh`, and `.ksh` files. Common shell control-flow keywords, strings, numbers, functions, and comments are highlighted automatically.

## Installation

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/logancammish/thre/main/install.sh | sh
```

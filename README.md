# thre

`thre` is a fast, friendly terminal file reader and editor for Linux. It is designed around visible, familiar shortcuts instead of modes: open a file, move with the arrow keys, type to edit, and press `Ctrl+S` to save.

The name is short for “The Third File Reader.” That is the only ceremony it needs.

## Highlights

- Responsive full-screen interface that adapts to the terminal
- Direct, modeless editing with safe unsaved-change prompts
- Syntax highlighting for Python, Rust, Java, Lua, Scala, C, C++, and shell scripts
- Five built-in themes plus user-defined themes and syntax definitions
- Soft wrapping, horizontal scrolling, line numbers, search, and go-to-line
- Automatic language detection with an explicit override
- Small dependency footprint and a single release binary
- Persistent user configuration using standard Linux paths

## Install

Install the latest Linux release into `~/.local/bin`:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/logancammish/thre/main/install.sh | sh
```

Set `THRE_INSTALL_DIR` to choose another destination. The installer currently
supports Linux on x86-64 and requires either `curl` or `wget`.

Alternatively, build the release binary with a current stable Rust toolchain:

```sh
cargo build --release
install -Dm755 target/release/thre ~/.local/bin/thre
```

Make sure `~/.local/bin` is on your `PATH`, then open any text file:

```sh
thre src/main.rs
```

Supplying a path that does not exist opens an empty buffer; it is created on the first save.

## Command line

```text
thre [OPTIONS] <FILE ...>

-t, --theme <NAME>       Select a built-in theme
-l, --language <LANG>    Override automatic syntax detection
    --no-line-numbers    Hide the line-number gutter
    --no-wrap            Use horizontal scrolling
    --list-themes        Print available theme names
-h, --help               Show command-line help
-V, --version            Show the version
```

## Controls

The shortcut strip is always visible by default, and `F1` opens the complete reference.

| Key | Action |
| --- | --- |
| Arrow keys | Move the cursor |
| Home / End | Start / end of line |
| Page Up / Page Down | Move one screen |
| Enter, Backspace, Delete | Edit normally |
| `Ctrl+S` | Save |
| `Ctrl+Q` | Quit; confirm if modified |
| `Ctrl+X` | Exit immediately without a warning |
| `Ctrl+O` / `Ctrl+N` | Open a file in a tab / create a new tab |
| `Ctrl+Tab` | Switch to the next tab |
| `F3` | Rename the current file |
| Mouse click | Place the editing cursor |
| Mouse wheel | Scroll while preserving the cursor position |
| `Shift` + mouse drag | Select text for terminal copying |
| `Ctrl+F` | Find text |
| `Ctrl+G` | Go to a line |
| `Ctrl+W` | Toggle soft wrapping |
| `Ctrl+L` | Toggle line numbers |
| `Ctrl+T` or `F2` | Cycle themes |
| `F1` or `?` | Show shortcut help |

## Configuration

Create `$XDG_CONFIG_HOME/thre/config`, or `~/.config/thre/config` when `XDG_CONFIG_HOME` is not set. The format is intentionally simple:

```ini
theme = ocean
language = auto
line_numbers = true
wrap = true
tab_width = 4
show_status = true
show_shortcuts = true
```

Command-line options take precedence over the configuration file. Unknown configuration keys are ignored, making configuration files forward-compatible.

Running `thre -t THEME` without a file updates the configured default and exits. Supplying both a theme and files applies it to that editing session.

### Custom themes

Add a file named `NAME.theme` to `~/.config/thre/themes` (or the equivalent
`$XDG_CONFIG_HOME` path), then select it with `thre --theme NAME`. Unspecified
colors inherit from Midnight. Colors use six-digit RGB values:

```ini
background = #101418
foreground = #e6edf3
muted = #7d8590
accent = #58a6ff
status_bg = #202830
status_fg = #ffffff
selection = #29384a
keyword = #ff7b72
string = #a5d6ff
comment = #8b949e
number = #79c0ff
type = #ffa657
function = #d2a8ff
```

Custom themes appear in `thre --list-themes` and in the `F2`/`Ctrl+T` theme cycle.

### Custom syntax highlighting

Add `NAME.syntax` under `~/.config/thre/syntaxes`. Definitions are intentionally
small and comma-separated:

```ini
name = Makefile DSL
extensions = mk, mak
line_comment = #
keywords = include, ifdef, ifndef, else, endif, define, endef
```

The `extensions` values can also be used with `--language`. Shell scripts are
built in and detected automatically for `.sh`, `.bash`, `.zsh`, and `.ksh` files.

## License

GPL-3.0-only. See [LICENSE](LICENSE).

Release history is recorded in [CHANGELOG.md](CHANGELOG.md).

## Making a release

Run `scripts/package-release.sh` on Linux x86-64, create the matching `0.2.0`
GitHub release, and upload both files produced in `dist/`. The archive name is
the stable filename used by `install.sh`.

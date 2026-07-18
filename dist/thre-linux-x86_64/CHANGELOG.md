# Changelog

All notable changes to `thre` are documented here. This project follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-07-18

### Added

- Custom themes loaded from the XDG `thre/themes` configuration directory
- Custom syntax definitions with configurable extensions, keywords, and line comments
- Built-in syntax highlighting for shell scripts, including `.sh`, `.bash`, `.zsh`, and `.ksh`
- Custom themes in `--list-themes` and the in-editor theme cycle

## [0.1.1] - 2026-07-18

### Added

- Click-to-position mouse editing across normal and wrapped lines
- Mouse-wheel document scrolling that preserves the cursor's screen position
- Terminal text selection and copying through `Shift`+drag

### Fixed

- Cursor and insertion-point rendering being offset by the line-number gutter
- Cursor placement around wide Unicode characters
- Scroll-wheel input moving the editing cursor unexpectedly

## [0.1.0] - 2026-07-18

### Added

- Modeless terminal editing with responsive wrapping and navigation
- Syntax highlighting for Python, Rust, Java, Lua, Scala, C, and C++
- Midnight, Graphite, Paper, Ember, and Ocean themes
- Multiple file tabs, in-editor file opening, new buffers, and file renaming
- Search, go-to-line, configurable line numbers, and XDG configuration
- Safe quitting with `Ctrl+Q` and immediate exit with `Ctrl+X`

[Unreleased]: https://github.com/logancammish/thre/compare/0.2.0...HEAD
[0.2.0]: https://github.com/logancammish/thre/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/logancammish/thre/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/logancammish/thre/releases/tag/0.1.0

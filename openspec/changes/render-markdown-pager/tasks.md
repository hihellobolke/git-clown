## 1. Renderer

- [x] 1.1 Add `pulldown-cmark`, `ratatui`, `crossterm`, `unicode-width` and the clap `env` feature
- [x] 1.2 `src/markdown.rs`: convert pulldown-cmark events to styled ratatui lines (headings, emphasis, code, lists, task lists, quotes and alerts, links, images, tables, rules, HTML reduced to text)
- [x] 1.3 Word-wrap to the screen width, keeping indentation on continuation lines; strip control characters
- [x] 1.4 Fit wide tables to the width by narrowing their widest columns and wrapping cells

## 2. Pager

- [x] 2.1 `src/pager.rs`: full-screen ratatui view with status bar, re-rendering on resize
- [x] 2.2 Keys: line, page and half-page scrolling, top/end, quit with `q`/`Esc`/`Ctrl-C`
- [x] 2.4 Sideways scrolling with a status bar hint for lines that still cannot fit
- [x] 2.3 Print the markdown unchanged when stdout is not a terminal or `TERM=dumb`

## 3. CLI

- [x] 3.1 `--readme` flag with `GIT_CLOWN_README` setting; show the README after any successful clone path
- [x] 3.2 `run_bare_clone` returns the worktree path so its README can be found
- [x] 3.3 `--docs` flag showing the embedded README; git URL optional with `--docs`
- [x] 3.4 Document the flags, keys, setting and fallback in `README.md`

## 4. Tests

- [x] 4.1 Unit tests for the renderer, pager state and drawing, plain fallback, CLI parsing and README lookup
- [x] 4.2 `cargo test` passes and `cargo clippy` reports no new warnings
- [x] 4.3 Manual check in tmux: `--docs`, `--readme`, `--bare --readme`, piped output, `GIT_CLOWN_README`

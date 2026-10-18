## Why

After cloning, the first thing a developer usually does is read the repository's README, which means opening it in an editor or a browser. git-clown's own documentation is likewise only readable as raw markdown. Rendering markdown in a terminal pager makes both one command away, without leaving the terminal.

## What Changes

- New `--readme` flag: after a successful clone (or when the repository already exists), show the checkout's `README.md` in a full-screen terminal pager. With `--bare`, the README of the default branch worktree is shown.
- New `GIT_CLOWN_README` environment variable: a setting that turns `--readme` on (`1`/`true`/`yes`/`on`) or off (`0`/`false`/`no`/`off`).
- New `--docs` flag: show git-clown's own README (embedded in the binary) in the pager and exit; no git URL is needed.
- The pager renders headings, emphasis, inline code, code blocks, lists and task lists, block quotes, links, images, tables and rules, re-wraps on resize, and is driven by less/vim style keys.
- When stdout is not a terminal (or `TERM=dumb`), the markdown is printed unchanged instead of opening the pager.

## Capabilities

### New Capabilities

- `markdown-pager`: Render markdown documents (a cloned README, git-clown's docs) in a scrollable terminal pager, with a plain-text fallback.

### Modified Capabilities

<!-- No existing spec-level requirements are changing. -->

## Impact

- **New modules**: `src/markdown.rs` (pulldown-cmark events to wrapped ratatui lines) and `src/pager.rs` (ratatui + crossterm pager, plain fallback).
- **`src/main.rs`**: new CLI flags; `run()` deduplicated so the README step runs once after any successful clone path; `run_bare_clone` returns the worktree path.
- **New dependencies**: `pulldown-cmark` (no default features), `ratatui` (crossterm backend only), `crossterm`, `unicode-width`; clap gains the `env` feature.
- **No breaking changes**: without the flags (or the variable) behaviour is unchanged. `git-clown` with no arguments still fails with the same "required arguments were not provided: <GIT_URL>" error.
- **Rollback plan**: revert the change; the flags and variable disappear and no files or config are left behind.
- **Affected teams**: git-clown users only; scripts that pipe git-clown output are unaffected unless they opt in, in which case they get plain markdown on stdout.
- **Testing plan**: unit tests for the renderer (each element, wrapping never exceeding the width, control characters stripped), the pager state (scroll bounds, quit keys, resize, a frame drawn on ratatui's `TestBackend`), the plain fallback, CLI parsing and README lookup; manual runs in a real terminal (tmux) and piped for `--docs`, `--readme` and `--bare --readme`.

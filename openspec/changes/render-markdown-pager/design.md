## Context

git-clown is a single-binary CLI in `src/main.rs`. It has no subcommands: the git URL is its only positional argument. The pager has to fit around that without changing how existing invocations behave.

## Goals / Non-Goals

**Goals:**
- Readable rendering of common README markdown (GitHub flavoured: tables, task lists, alerts, strikethrough).
- Opt-in only; never block or change output for scripts.
- Safe on untrusted content: a cloned README must not be able to drive the terminal.

**Non-Goals:**
- Search inside the pager, syntax highlighting of code blocks, clickable links.
- Rendering non-markdown READMEs (`README`, `README.rst`).

## Decisions

### D1 — Wrap lines ourselves instead of using `Paragraph::wrap`

The renderer takes the screen width and word-wraps each block itself, so continuation lines keep their list indentation, quote bar or code gutter. The pager re-renders when the width changes. This also gives an exact line count for scroll bounds without ratatui's unstable `rendered-line-info` feature.

### D2 — Plain fallback prints the markdown source

When stdout is not a terminal, the original markdown is written unchanged: it is already plain text, it is what tools downstream expect, and it is lossless.

### D3 — `--docs` shows the embedded README

`README.md` is included with `include_str!`, so the docs always match the binary. `--docs` makes the git URL optional (`required_unless_present`) and conflicts with `--bare` and the URL, but not with `--readme`, so a globally exported `GIT_CLOWN_README` does not break it.

### D4 — Control characters are stripped

Text from the document goes through `clean()`: tabs become four spaces and every other control character is dropped, so escape sequences in a README cannot reach the terminal.

### D5 — README failures do not fail the clone

A missing README, a read error or a pager error is logged as a warning; the exit status stays the clone's.

## Flow

```
user ──> git-clown --readme <url>
           │ clone / bare clone / already exists ──> (status, checkout path)
           │ status != 0 ──> exit with status
           ▼
         find_readme(checkout) ── none ──> warn, exit with status
           │
           ▼ spawn_blocking
         pager::show(title, markdown)
           │ stdout not a TTY ──> print markdown, return
           ▼
         ratatui::try_init ─> loop { draw (re-render on resize) ; read key } ─> q ─> try_restore
```

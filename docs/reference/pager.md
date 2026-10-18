# Markdown pager

The pager shows markdown full-screen in the terminal. It is used by
`git-clown --readme <url>` (or `GIT_CLOWN_README`) for the cloned repository's
README, and by `git-clown --docs` for git-clown's own README.

## When it opens

| Condition | Result |
|-----------|--------|
| stdout is a terminal and `TERM` is not `dumb` | The pager opens. git-clown waits until you quit it. |
| stdout is piped or redirected, or `TERM=dumb` | The markdown source is printed to stdout unchanged (with a final newline added if missing). |

With `--readme`, the pager only opens after a successful clone, or when the
repository was already cloned.

## Which README

git-clown looks at the top of the checkout (for `--bare`, the default-branch
worktree) for these files, in order, and uses the first that is a regular
file:

1. `README.md`
2. `README.markdown`
3. `readme.md`
4. `Readme.md`

If none exists, it logs `No markdown README found in <dir>` as a warning. If
the file can't be read or the pager fails, it logs a warning too. In both cases
the exit status stays that of the clone.

## Keys

| Key | Action |
|-----|--------|
| `q`, `Esc`, `Ctrl-C` | Quit |
| `↓`, `j`, `Enter` | Down one line |
| `↑`, `k` | Up one line |
| `Space`, `f`, `PageDown` | Down one page |
| `b`, `PageUp` | Up one page |
| `d` | Down half a page |
| `u` | Up half a page |
| `g`, `Home` | Top |
| `G`, `End` | End |
| `→`, `l` | Right 8 columns (only when lines are wider than the screen) |
| `←`, `h` | Left 8 columns |

Other keys are ignored.

## Status bar

The last screen row, in reverse video:

```text
 <title>  <first>-<last>/<total> (<percent>%)  [←→ h l: more to the side]  q quit · ↑↓ j k scroll · space b page · g G top/end
```

- `<title>`: the README's path relative to the workspace root (for example
  `github.com/example/repo/README.md`), or `git-clown` for `--docs`.
- `<first>-<last>/<total> (<percent>%)`: the lines shown, the number of lines,
  and how far through the document the bottom of the screen is.
- `←→ h l: more to the side` appears only when some lines are wider than the
  screen.

## Rendering

Markdown is parsed as CommonMark with GitHub extensions: tables, task lists,
strikethrough and alerts.

| Element | Shown as |
|---------|----------|
| Heading level 1 | Bold magenta, underlined with `═` |
| Heading level 2 | Bold cyan, underlined with `─` |
| Heading level 3 | Bold blue |
| Heading level 4–6 | Bold, underlined |
| *Emphasis*, **strong**, ~~strikethrough~~ | Italic, bold, crossed out |
| `Inline code` | Yellow |
| Code block | Yellow, indented by two columns, original line breaks and indentation kept; tabs become four spaces. The language of a fenced block is shown above it in dim italic. |
| Bulleted list | `•`, `◦`, `▪` by nesting depth; wrapped lines are indented under the text |
| Numbered list | Numbers counted from the list's start number |
| Task list item | `[x]` in green, `[ ]` dimmed |
| Block quote | `┃ ` in front of every line, grey italic |
| Alert (`> [!NOTE]`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`) | A block quote headed by the alert's name in bold |
| Link | Blue underlined text followed by the URL in `<…>`. The URL is left out for `#anchor` links and when the text is the URL itself. A link with no text shows the URL. |
| Image | `[image: <alt text>]`, dimmed |
| Table | Columns separated by ` │ `, a `─┼─` rule under the header row, cells aligned as the table specifies |
| Horizontal rule | A dim `─` line across the text width |
| HTML | Tags and comments removed; the remaining text is shown dimmed. `<img alt="…">` becomes `[image: …]`. |

Blocks are separated by a blank line. Text is word-wrapped to the screen width;
a word longer than the line is split. The document is wrapped again when the
terminal is resized.

### Wide tables

A table that fits keeps its natural column widths. A wider table keeps its
narrow columns and shares the remaining width among the wide ones, wrapping
cells inside their column, so a row can take several lines. Each column gets
at least one character, so only a table with more columns than the screen can
hold stays wider than the screen; then the status bar shows
`←→ h l: more to the side` and the pager scrolls sideways.

### Control characters

Tabs are expanded and all other control characters are removed from the text
before it is drawn, so a document can't send escape sequences to the terminal.
This applies only to the pager; the plain-text fallback prints the file
unchanged.

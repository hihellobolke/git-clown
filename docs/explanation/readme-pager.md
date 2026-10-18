# The README pager

When you clone a repository, the first thing you usually do is read its
README. `--readme` shows it right there in the terminal, formatted. This page
explains the choices behind how that works.

## Opt-in, and invisible to scripts

Opening a full-screen view after a clone is useful when you're at the keyboard
and a nuisance anywhere else. So the pager only runs when you ask for it, with
`--readme` or `GIT_CLOWN_README`, and only when stdout is a terminal.

When stdout is a pipe or a file, git-clown prints the markdown exactly as it is
in the file. Markdown is already readable as plain text, it's what any tool
downstream expects, and nothing is lost. That's why
`git-clown --docs > git-clown.md` gives you the real file, not a rendering of
it.

## The clone comes first

The README is extra. A missing README, one that can't be read, or a terminal
that can't show the pager is reported as a warning, and git-clown still exits
with the clone's status. A failed clone never opens the pager, so you don't
end up reading an old README while the clone you asked for didn't happen. And
the shell integration still takes you into the repository once you close the
pager.

## Rendering for the terminal

The renderer formats what READMEs actually contain: GitHub-flavoured headings,
lists, code, tables, task lists and alerts. It wraps text itself rather than
leaving it to the terminal, so that a wrapped list item stays indented under
its bullet and a wrapped quote keeps its bar. When you resize the terminal, the
document is laid out again for the new width.

Tables get special care, because a single long cell would otherwise push the
rest of the table off screen. Narrow columns keep their width, wide ones share
what's left and wrap inside their column. Only a table with more columns than
the screen has room for is still too wide, and then the pager lets you scroll
sideways rather than cutting anything off.

There are deliberate gaps: no search, no syntax highlighting, no clickable
links, and only markdown READMEs. For more than reading, pipe the text into
your own tools.

## A README is untrusted input

You may clone a repository you have never seen before, and its README goes
straight to your terminal. Terminals interpret escape sequences, which can
change colours, move the cursor, rewrite the window title and more. The
renderer removes every control character (tabs become spaces) before drawing
anything, so a README can only ever show text.

## `--docs` is the README in the binary

git-clown's `README.md` is compiled into the binary, and `--docs` shows that
copy. The documentation you read with `--docs` always describes the version
you are running, even with no network and no source checkout. The pages under
`docs/` go into more depth, but they are not built in.

# How the shell integration works

`git clown <url>` can leave your shell inside the cloned repository. That
sounds like a small convenience, but no ordinary program can do it. This page
explains why, how git-clown gets around it, and why the doctor checks what it
checks.

## A program can't change your shell's directory

Every process has its own working directory, and a child process inherits a
copy of its parent's. git-clown is a child of your shell (or of git, which is a
child of your shell). It can change its own directory as often as it likes;
the shell never sees it. When git-clown exits, the shell is exactly where it
was.

Only the shell can change the shell's directory, so the shell has to run the
`cd` itself. Tools such as zoxide and direnv have the same problem and solve it
the same way: a shell function, loaded from the rc file, that runs the program
and then acts on what it reports.

## Handing the path back through a file

The function has to learn *which* directory to go to. The obvious channel,
having the program print the path and the function capture it, would break
git-clown's output: git's messages and the README pager use the same stdout.
You'd lose them, or the function would have to guess which line is the path.

Instead, the `git-clown` function creates a temporary file and passes its name
in `GIT_CLOWN_CD_FILE`. stdout and stderr stay connected to your terminal as
usual. Only when the clone succeeds does the binary write the repository's
path into the file; the function then reads it, changes directory, and removes
the file. A failed clone writes nothing, so the shell doesn't move and the
exit status is git's.

The path is written as raw bytes with no newline, and the function reads the
whole file, so even unusual directory names (with spaces or newlines) arrive
intact.

## Wrapping `git` too

Most people type `git clown`, not `git-clown`. git finds `git-clown` on `PATH`
and runs it as a separate process, which again can't move the shell. So the
integration also defines a `git` function: when the first argument is `clown`,
it calls the `git-clown` function; anything else goes straight to the real git
with `command git`. Ordinary git commands behave exactly as before.

The cost is that there can only be one shell function called `git`. If you
have your own, the one loaded last wins, and you have to chain them by hand.

## Why the rc file runs `git-clown init`

The rc block doesn't contain the functions themselves. It runs
`eval "$(command git-clown init zsh)"`, which asks the installed binary for
them each time a shell starts. When you upgrade git-clown, new shells pick up
the matching functions without touching the rc file. The block also checks
that `git-clown` is on `PATH` first, so uninstalling the binary doesn't break
shell startup.

## What the doctor trusts

Setting this up involves several things that can each go wrong: the binary
must be on `PATH`, the rc file must contain the block, the shell must be able
to read the rc file, and the functions must actually work. The doctor checks
each one, and is careful about how it knows.

- **It only recognises its own block in the rc file.** Deciding whether a
  hand-written `eval` line is live or commented out (perhaps inside quotes or
  a command substitution) would take a full shell parser. Rather than guess,
  the doctor looks only for its marked block.
- **It looks for runtime evidence.** The wrapper sets `GIT_CLOWN_SHELL` when it
  runs the binary. If the doctor sees it, the integration is loaded in the
  shell you ran it from, however you set it up. That is how a hand-written
  setup is recognised.
- **It lets the shell check the rc file.** A syntax error anywhere in an rc
  file stops the shell from reading the rest of it, including the block. The
  doctor runs `bash -n` or `zsh -n`, which parses the file without running
  anything.
- **It tests the real thing.** The self-test starts a clean shell, loads the
  functions from `git-clown init`, and checks that both `git-clown` and
  `git clown` really change directory. It exercises the same code your shell
  uses, without touching your rc files.

The report is plain text, not a full-screen view, so it stays in your
scrollback and can be piped or pasted into a bug report. The only interactive
part, the `[y/N]` question, appears only when a person is at the terminal.

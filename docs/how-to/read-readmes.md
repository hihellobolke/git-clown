# Read READMEs in the terminal

git-clown can show a repository's README, formatted, in a full-screen pager
right after cloning it. This guide shows the ways to use that.

## Show the README after one clone

```sh
git clown --readme https://github.com/example/repo.git
```

If the repository is already cloned, git-clown skips the clone and shows the
README straight away, so this also works as "open the README of a repository I
have".

Press `q` to close the pager. With the shell integration loaded, your shell
then changes into the repository.

For a `--bare` clone, the README comes from the default branch's worktree:

```sh
git clown --bare --readme https://github.com/example/repo.git
```

## Always show the README

Set `GIT_CLOWN_README` in your shell's rc file:

```sh
export GIT_CLOWN_README=1
```

To skip it for one clone while it is set:

```sh
GIT_CLOWN_README=0 git clown https://github.com/example/repo.git
```

## Read git-clown's own documentation

```sh
git-clown --docs
```

This shows git-clown's README, which is built into the binary, so it always
matches the version you have installed.

## Save or search the text instead

When the output is not a terminal, git-clown prints the markdown source
unchanged instead of opening the pager. Use that to keep a copy, or to read it
with your own tools:

```sh
git-clown --docs > git-clown.md
git-clown --docs | less
git-clown --docs | grep -i workspace
```

Setting `TERM=dumb` has the same effect.

## If no README appears

git-clown looks for `README.md`, `README.markdown`, `readme.md` and
`Readme.md`, in that order, at the top of the checkout. If none is there (for
example, the repository only has a plain `README` or a `README.rst`), it logs a
warning and does nothing else:

```text
[WARN  git_clown] No markdown README found in /home/you/ws/github.com/example/repo
```

If the clone fails, the pager doesn't open at all.

For the keys and what the pager can display, see the
[pager reference](../reference/pager.md).

# Change the workspace root

git-clown puts every repository under one directory, the workspace root. It is
`~/ws` unless you set `WORKSPACE_ROOT`. This guide shows how to use a different
directory.

## Set it for every shell

Add an export to your shell's rc file (`~/.zshrc`, `~/.bashrc`, or
`~/.bash_profile` on macOS):

```sh
export WORKSPACE_ROOT="$HOME/src"
```

Open a new terminal, or `source` the file, then check that git-clown sees it:

```sh
git-clown doctor
```

```text
  ✓ workspace root     ~/src (from $WORKSPACE_ROOT)
```

Use an absolute path. Repositories you clone from now on go under it, for
example `~/src/github.com/example/repo`.

## Set it for one command

```sh
WORKSPACE_ROOT=/mnt/scratch git clown https://github.com/example/repo.git
```

## Move existing clones

git-clown does not move repositories you cloned earlier. To keep using them
with git-clown, move the whole tree to the new root, keeping the
`<host>/<owner>/<repo>` structure:

```sh
mv ~/ws/* "$WORKSPACE_ROOT"/
```

Move any `.envrc` or `.env` files you keep in the workspace along with them
(the `*` above does not match hidden files at the top level):

```sh
mv ~/ws/.envrc ~/ws/.env "$WORKSPACE_ROOT"/ 2>/dev/null
```

Afterwards, `git clown <url>` finds the moved repositories as already cloned.

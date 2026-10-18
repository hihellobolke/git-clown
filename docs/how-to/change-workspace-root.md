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
`<host>/<owner>/<repo>` structure. Plain clones keep working after the move;
clones made with `--bare` need one more step, below.

1. Create the new root, then move everything into it:

   ```sh
   mkdir -p "$WORKSPACE_ROOT"
   mv ~/ws/* "$WORKSPACE_ROOT"/
   ```

2. Move any `.envrc` or `.env` files you keep at the top of the workspace (the
   `*` above does not match hidden files):

   ```sh
   mv ~/ws/.envrc ~/ws/.env "$WORKSPACE_ROOT"/ 2>/dev/null
   ```

3. If you have `--bare` clones, repair their worktrees. git records the
   location of each worktree as an absolute path, so after the move git
   commands in a worktree fail (with an error that the worktree's `.git` file
   does not point to a valid repository) until git is told the new locations. This finds every `--bare` clone under
   the new root and runs `git worktree repair` for each of its worktrees (the
   `grep` skips the `.git` files of submodules):

   ```sh
   find "$WORKSPACE_ROOT" -mindepth 4 -maxdepth 4 -type d -name .bare |
   while IFS= read -r bare; do
     repo=$(dirname "$bare")
     find "$repo" -mindepth 2 -type f -name .git -not -path "$bare/*" |
     while IFS= read -r gitfile; do
       grep -q '/\.bare/worktrees/[^/]*$' "$gitfile" &&
         git -C "$repo" worktree repair "$(dirname "$gitfile")"
     done
   done
   ```

   git prints a `repair:` line for each link it fixes. Check a worktree
   afterwards, for example:

   ```sh
   git -C "$WORKSPACE_ROOT/github.com/acme/widgets/main" status
   ```

   Running `git clown --bare <url>` again does not repair worktrees; it skips
   the ones that already exist.

Afterwards, `git clown <url>` finds the moved repositories as already cloned.

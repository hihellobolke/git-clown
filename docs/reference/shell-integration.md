# Shell integration

The shell integration makes the shell change into the repository after a
successful `git clown <url>`. It supports bash and zsh.

For setup steps, see [Set up the shell integration](../how-to/set-up-shell-integration.md).
For the reasons behind the design, see
[How the shell integration works](../explanation/shell-integration.md).

## Loading

```sh
eval "$(command git-clown init bash)"   # or zsh
```

`git-clown init <shell>` prints the script (the source is
[`shell/git-clown.sh`](../../shell/git-clown.sh)); the only difference between
the two shells is the shell name filled in.

## Functions

### `git-clown`

Wraps the `git-clown` binary.

1. Create a temporary file with `mktemp "${TMPDIR:-/tmp}/git-clown-cd.XXXXXX"`.
   If that fails, run the binary without the integration and return its status.
2. Run `command git-clown "$@"` with `GIT_CLOWN_CD_FILE` set to the file and
   `GIT_CLOWN_SHELL` set to the shell name.
3. If the binary exited with `0` and the file is not empty, read the whole file
   as a directory path. If that directory exists, `cd` into it.
4. Remove the file and return the binary's exit status (or the status of a
   failed `cd`).

stdin, stdout and stderr pass through unchanged.

### `git`

```text
git clown <args>   →  git-clown <args>     (the function above)
git <anything>     →  command git <anything>
```

Only a first argument of exactly `clown` is redirected.

Both functions work with `set -u` and `set -e`, and with an empty argument
list, also in bash 3.2.

## When the shell changes directory

| Command | Directory afterwards |
|---------|----------------------|
| Successful clone | `<workspace root>/<host>/<owner>/<name>` |
| Repository already cloned | The existing clone |
| `--bare`, successful | The default-branch worktree |
| `--readme` | As above, once the pager is closed |
| Failed clone, invalid URL, any non-zero exit | Unchanged |
| `--docs`, `doctor`, `init`, `--help` | Unchanged |

## Handoff file

The binary writes the target only when `GIT_CLOWN_CD_FILE` is set and not
empty. It writes the absolute path as raw bytes with no line ending, so a path
containing newlines (even a trailing one) arrives intact. When the variable is
not set, the binary logs a tip about `git-clown doctor --install` instead.

## Rc file block

`git-clown doctor --install` writes this block (shown for zsh):

```sh
# >>> git-clown shell integration >>>
# Lets `git clown <url>` cd into the cloned repo. Managed by `git-clown doctor --install`.
if command -v git-clown >/dev/null 2>&1; then
  eval "$(command git-clown init zsh)"
fi
# <<< git-clown shell integration <<<
```

| Shell | Rc file |
|-------|---------|
| zsh | `$ZDOTDIR/.zshrc`, or `~/.zshrc` |
| bash, Linux | `~/.bashrc` |
| bash, macOS | `~/.bash_profile` |

Install rules:

| Rc file contains | `--install` does |
|------------------|------------------|
| No block (or no file) | Appends the block after a blank line; creates the file and its directory if needed. |
| The current block | Nothing. |
| A different block between the markers | Replaces it in place. |

Lines outside the markers are never read or changed. A hand-written
`eval "$(git-clown init …)"` line is not recognised as an installed block.

## Limitations

- Only bash and zsh. Other shells (fish, for example) are reported as
  unsupported by the doctor.
- The `git` function replaces any `git` function defined before it; one defined
  after it replaces the integration's.

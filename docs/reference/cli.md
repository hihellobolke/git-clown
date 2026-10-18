# Command reference

```text
git-clown [--bare] [--readme] <GIT_URL>
git-clown --docs
git-clown doctor [--install] [--shell <SHELL>]...
git-clown init <SHELL>
git-clown help [COMMAND]
git-clown --help | --version
```

Every form also works as `git clown …`: git runs any executable called
`git-<name>` on `PATH` as the subcommand `git <name>`.

## Clone: `git-clown <GIT_URL>`

Clones `GIT_URL` into the workspace. The target directory is derived from the
URL; see [Workspace layout](workspace-layout.md).

| Argument / option | Description |
|-------------------|-------------|
| `<GIT_URL>` | URL of the repository to clone. Required, unless `--docs` is given. See [Supported URLs](workspace-layout.md#supported-urls). |
| `--bare` | Clone as a bare repository and add a worktree for the default branch. See [Bare layout](workspace-layout.md#bare-layout). |
| `--readme` | After a successful clone, show the repository's README in the [pager](pager.md). Also set by [`GIT_CLOWN_README`](configuration.md#environment-variables). |

Steps, in order:

1. Determine the workspace root (`$WORKSPACE_ROOT`, else `~/ws`, which is
   created if missing).
2. Parse the URL into host, owner and repository name.
3. Create `<root>/<host>/<owner>`.
4. Find the nearest [environment file](configuration.md#environment-files) and
   merge it over the current environment.
5. Without `--bare`: if `<root>/<host>/<owner>/<name>/.git` is a directory,
   skip the clone; otherwise run `git clone <GIT_URL> <name>` in
   `<root>/<host>/<owner>`.
   With `--bare`: run the [bare setup](workspace-layout.md#bare-layout).
6. On success, report the repository directory to the
   [shell integration](shell-integration.md) if it is loaded; otherwise log a
   tip about `git-clown doctor --install`.
7. On success with `--readme`, show the README.

All git commands run with the merged environment from step 4.

**Output.** git-clown's own log lines go to stderr. git's stdout and stderr are
captured and printed (to stdout and stderr respectively) once the git command
has finished, so git shows no live progress.

**Exit status.**

| Status | Meaning |
|--------|---------|
| `0` | Cloned, or already cloned. With `--bare`: the whole setup succeeded. |
| git's status | `git clone` failed (plain clone only). |
| `1` | Any other error: an unparseable URL, a directory that can't be created, an unreadable environment file, a failed step of the `--bare` setup, or git killed by a signal. |
| `2` | Invalid command line. |

A missing README, or a pager error, is logged as a warning and does not change
the exit status.

## `git-clown --docs`

Shows git-clown's own `README.md`, embedded in the binary at build time, in the
[pager](pager.md), and exits. It cannot be combined with `<GIT_URL>` or
`--bare`. `--readme` (or `GIT_CLOWN_README`) is accepted and ignored.

Exit status: `0`, or `1` if the pager fails.

## `git-clown doctor`

Checks the installation and the shell integration, and can install the
integration. See the [doctor reference](doctor.md) for every check.

| Option | Description |
|--------|-------------|
| `--install` | Install the shell integration into each selected shell's rc file, or update an outdated block, before running the checks. Does nothing to a file that already has the current block. |
| `--shell <SHELL>` | Shell to check or install for: `bash` or `zsh`. Repeat for several. Default: the shell named by `$SHELL`. |

Exit status: `0` when no check failed, `1` otherwise.

## `git-clown init <SHELL>`

Prints the shell integration script for `<SHELL>` (`bash` or `zsh`) to stdout.
It is meant to be evaluated by the shell:

```sh
eval "$(command git-clown init zsh)"
```

See [Shell integration](shell-integration.md) for what the script defines.
Exit status: `0`.

## `git-clown help`, `--help`, `--version`

`git-clown help [COMMAND]` and `-h`/`--help` print usage; `-V`/`--version`
prints the version.

Without the shell integration, `git clown --help` is turned by git into
`git help clown`, which looks for a man page and fails. Use
`git-clown --help`. With the integration loaded, `git clown --help` works.

## Internal: `git-clown __selftest-cd <DIR>`

Hidden from `--help`. Reports `<DIR>` to the shell integration exactly as a
successful clone would. The doctor's self-test uses it. Exits `1` when
`GIT_CLOWN_CD_FILE` is not set.

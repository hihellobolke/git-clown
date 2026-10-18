# Configuration

git-clown has no configuration file. It is configured through environment
variables, and passes extra variables to git from environment files in the
workspace.

## Environment variables

### Read by git-clown

| Variable | Effect | Default |
|----------|--------|---------|
| `WORKSPACE_ROOT` | Directory that all clones go under. See [Workspace layout](workspace-layout.md). | `~/ws` |
| `GIT_CLOWN_README` | When true, show the README after every clone, as if `--readme` were given. True: `1`, `true`, `yes`, `on`, `y`, `t`. False: `0`, `false`, `no`, `off`, `n`, `f`. Case does not matter. Any other value, including an empty one, is a command-line error (exit status 2). | false |
| `RUST_LOG` | Log level of git-clown's own messages on stderr: `error`, `warn`, `info`, `debug` or `trace`. `RUST_LOG=warn` hides the step-by-step `INFO` lines. | `info` |
| `TERM` | `dumb` makes the pager print plain markdown instead of opening. | |
| `SHELL` | The doctor uses its file name (`bash`, `zsh`) as the default shell. | |
| `ZDOTDIR` | Directory of the zsh rc file (`$ZDOTDIR/.zshrc`). | `~` |
| `HOME` | Location of `~`: the default workspace root and the rc files. | |

### Set by the shell integration

You don't set these yourself. The integration's `git-clown` function sets them
for the one command it runs.

| Variable | Meaning |
|----------|---------|
| `GIT_CLOWN_CD_FILE` | Path of a temporary file. After a successful clone, git-clown writes the absolute repository directory into it. |
| `GIT_CLOWN_SHELL` | Name of the shell running the wrapper (`bash` or `zsh`). The doctor uses it to report that the integration is loaded. |

The wrapper also uses `TMPDIR` (default `/tmp`) for the temporary file.

### Passed to git

git runs with git-clown's own environment, plus the variables from the nearest
environment file (below). Any variable git understands (`GIT_SSH_COMMAND`,
`GIT_TRACE`, `https_proxy`, …) can be set either way.

## Environment files

Before running git, git-clown looks for an environment file and adds its
variables to the environment of every git command it runs for that clone.

### Lookup

1. Start in `<workspace root>/<host>/<owner>`, the directory the repository is
   cloned into.
2. If the directory contains a file `.envrc`, use it. Otherwise, if it contains
   a file `.env`, use it.
3. Otherwise stop if this is the workspace root, or go to the parent directory
   and repeat step 2.

Only the first file found is used. If none is found, git-clown logs a warning
and uses the current environment unchanged.

Values from the file replace variables of the same name in the current
environment. The file's variables only reach git; git-clown's own settings
(such as `WORKSPACE_ROOT`) are read before the file is.

### `.envrc` format

- Each line is trimmed. Empty lines and lines starting with `#` are skipped.
- A line `NAME=VALUE` or `export NAME=VALUE` sets `NAME`. `NAME` starts with a
  letter or `_` and contains letters, digits and `_`.
- `VALUE` is the rest of the line. If it starts and ends with the same quote
  (`"` or `'`), that pair of quotes is removed.
- Nothing else is interpreted: no variable expansion, no escapes, no inline
  comments, no commands. Other lines are ignored.

The file is read, not run, so a direnv-style `.envrc` that calls functions
(`use`, `layout`, `source_env`) only contributes its plain assignments.

### `.env` format

Parsed by the [dotenvy](https://crates.io/crates/dotenvy) crate: `KEY=VALUE`
lines, with quoting and `$VAR` / `${VAR}` expansion. A line that can't be parsed
is skipped with a warning.

### Errors

If the chosen file can't be opened, the clone is aborted with exit status 1.

## Rc file

`git-clown doctor --install` writes a block into your shell's rc file. See
[Shell integration](shell-integration.md#rc-file-block).

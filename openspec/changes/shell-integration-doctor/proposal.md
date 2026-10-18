## Why

After `git clown <url>` the user is still in the directory they started from and has to `cd` to `$WORKSPACE_ROOT/<host>/<org>/<repo>` by hand. A child process cannot change its parent shell's working directory, so this needs shell integration, plus a way to see whether that integration is set up.

## What Changes

- New `git-clown init <bash|zsh>` prints shell functions that wrap `git-clown` and `git clown` and `cd` into the repository after a successful run.
- The binary reports the repository directory through a dedicated channel: the file named by `GIT_CLOWN_CD_FILE`, which the wrapper sets. stdout, stderr and exit codes are unchanged.
- New `git-clown doctor` checks git, `git-clown` on PATH, the workspace root, the detected shell, the rc file, and runs an end-to-end self-test of the wrapper. `--install` (or answering `y` at the prompt) adds a marked block to the rc file; running it again changes nothing.
- `run_bare_clone` returns the worktree path. `run()` is simplified to a single flow.

## Capabilities

### New Capabilities

- `shell-integration`: shell functions plus the `GIT_CLOWN_CD_FILE` channel that let the shell `cd` into the clone.
- `doctor`: setup checks and an idempotent installer for the shell integration.

### Modified Capabilities

<!-- None: clone behaviour is unchanged apart from writing the cd target file when asked to. -->

## Impact

- **Code**: `src/shell.rs`, `src/doctor.rs` and `shell/git-clown.sh` (new); `src/main.rs` (subcommands, clone flow).
- **Dependencies**: `crossterm` (single-key `[y/N]` prompt); `tempfile` moves from dev-dependencies to dependencies (doctor self-test).
- **Affected teams/users**: everyone using git-clown from bash or zsh. The change is opt-in: nothing changes until the integration is installed.
- **Rollback plan**: users remove the `# >>> git-clown shell integration >>>` block from their rc file. If they keep the block and downgrade, the older binary treats `init` as a git URL and prints a parse error at every shell startup, so the block has to be removed.
- **Testing plan**: unit tests for rc-block upsert (add, unchanged, update; lines the user wrote never block install), shell detection and report rendering. Integration tests in `tests/shell_integration.rs` run real bash/zsh: clone, already cloned, `--bare`, failed clone (no cd, exit code kept), non-`clown` git passthrough, `doctor --install` idempotency, rc loaded by an interactive shell, and doctor seeing an active session.

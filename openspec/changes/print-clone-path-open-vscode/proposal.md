## Why

After cloning, `git-clown` exits silently with no indication of where the repository landed on disk. Developers are left to reconstruct the path from the URL themselves. Adding a clear summary line at the end of a successful run removes that friction and makes the tool feel complete.

## What Changes

- After a successful standard clone, print the absolute path of the cloned repository directory to stdout.
- After a successful bare clone, print the absolute path of the initial worktree directory (e.g. `$WORKSPACE_ROOT/<host>/<org>/<repo>/<branch>/`) to stdout.
- After printing the path, prompt the user: **"Open in VS Code? [Y/n] (auto-closing in 5 s)"**.
  - If the user presses Enter or types `y`/`Y` within 5 seconds, run `code <path>`.
  - If the user types `n`/`N`, or 5 seconds elapse with no input, skip silently.

## Capabilities

### New Capabilities

- `clone-result-output`: Print the resulting clone or worktree directory to stdout after a successful operation.
- `vscode-open-prompt`: Interactively ask the user whether to open the result directory in VS Code, with a 5-second timeout that defaults to no-op.

### Modified Capabilities

<!-- No existing spec-level requirements are changing. -->

## Impact

- **`src/main.rs`**: `run()` and `run_bare_clone()` return (or surface) the result path; new prompt logic added after clone success.
- **No new dependencies**: timeout-based stdin reading can be implemented with `tokio::time::timeout` and `tokio::io` (already available via `tokio = { features = ["full"] }`).
- **No breaking changes**: new output is additive; prompt auto-dismisses so non-interactive use (piped stdout) is unaffected by the 5-second timeout.
- **Rollback plan**: revert the two call sites in `run()` / `run_bare_clone()` plus the prompt function; no data or config changes involved.
- **Affected users**: all `git-clown` users benefit immediately; no migration required.
- **Testing plan**: unit-test path formatting for both clone modes; integration-test the 5-second timeout path (no input → no `code` invocation); mock `code` binary in CI to verify the open path.

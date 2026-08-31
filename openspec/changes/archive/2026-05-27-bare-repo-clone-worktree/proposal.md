## Why

Developers working with multiple branches simultaneously need a way to avoid constant branch switching. Git worktrees solve this, but the setup is tedious and error-prone. `git-clown` should support cloning bare repositories and automatically wiring up the standard worktree layout so developers get a ready-to-use multi-branch workspace in one command.

## What Changes

- Add `--bare` flag to the `git-clown` CLI to trigger bare clone + worktree setup mode.
- When `--bare` is used, clone the repository as a bare repo into `$WORKSPACE_ROOT/github.com/<org>/<repo>/.bare` instead of the normal clone destination.
- Create a `.git` file at `$WORKSPACE_ROOT/github.com/<org>/<repo>/.git` containing `gitdir: ./.bare` so git tools recognize the directory as a repo.
- Fix the remote fetch refspec via `git config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*"` (bare clones omit this by default).
- Run `git fetch` to populate remote-tracking refs.
- Detect the default branch (preferring `main`, falling back to `master`, then whatever `HEAD` points to).
- Add the default branch as a worktree at `$WORKSPACE_ROOT/github.com/<org>/<repo>/<branch>/`.
- Create a VS Code `.code-workspace` file at `$WORKSPACE_ROOT/github.com/<org>/<repo>/<repo>.code-workspace` so the whole repo directory (with all worktrees visible) opens cleanly in VS Code.

## Capabilities

### New Capabilities

- `bare-clone`: Clone a repository as a bare repo into the structured workspace layout and wire up the `.git` pointer file.
- `worktree-init`: Detect the default branch and add it as the initial git worktree under the repo directory.
- `vscode-workspace`: Generate a `<repo>.code-workspace` file that opens the entire repo directory (including all worktrees) in VS Code.

### Modified Capabilities

<!-- No existing specs are being changed. -->

## Impact

- **`src/main.rs`**: New `--bare` CLI flag; new code paths for bare clone, `.git` file creation, refspec fix, fetch, default-branch detection, worktree add, and `.code-workspace` generation.
- **`Cargo.toml`**: No new dependencies expected; existing `tokio`, `clap`, `anyhow`, `colored`, and `dotenvy` are sufficient.
- **Rollback plan**: The `--bare` flag is purely additive. Removing it restores previous behaviour. Directories created by a failed bare-clone run can be deleted manually; no shared state is modified.
- **Affected teams/users**: CLI users who want worktree-based workflows.
- **Testing plan**:
  - Unit tests for default-branch detection logic (mock `HEAD` ref).
  - Integration test: run `git-clown --bare` against a local bare test repo; assert directory structure, `.git` file content, worktree existence, and `.code-workspace` file content.
  - Manual smoke test against a real GitHub repo.

## 1. CLI Flag

- [x] 1.1 Add `--bare` boolean flag to the `Args` struct in `src/main.rs` using `clap`
- [x] 1.2 Thread the `bare` flag value through `main` into the clone dispatch logic

## 2. Bare Clone

- [x] 2.1 Add `clone_bare` async function that runs `git clone --bare <url> <repo_root>/.bare` with inherited env vars
- [x] 2.2 Check if `.bare` already exists before cloning; skip and log if it does
- [x] 2.3 Write the `.git` pointer file at `<repo_root>/.git` with content `gitdir: ./.bare`

## 3. Refspec Fix & Fetch

- [x] 3.1 Run `git -C .bare config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*"` after bare clone
- [x] 3.2 Run `git -C .bare fetch` to populate `refs/remotes/origin/*`

## 4. Default Branch Detection

- [x] 4.1 Add `detect_default_branch` async function that calls `git -C .bare symbolic-ref HEAD` and parses the branch name from the ref output
- [x] 4.2 Return an error with a clear message if the ref cannot be resolved (e.g., empty repo)

## 5. Worktree Setup

- [x] 5.1 Add `add_worktree` async function that runs `git worktree add <branch>` from the repo root directory
- [x] 5.2 Check if the worktree directory already exists before running `git worktree add`; skip and log if it does

## 6. VS Code Workspace File

- [x] 6.1 Add `write_vscode_workspace` function that serialises `{"folders":[{"path":"."}],"settings":{}}` to `<repo_root>/<repo>.code-workspace`
- [x] 6.2 Skip writing if the file already exists; log an informational message

## 7. Integration & Wiring

- [x] 7.1 Add `run_bare_clone` orchestration function that calls steps 2–6 in order, propagating errors
- [x] 7.2 In `main`, branch on `args.bare`: call `run_bare_clone` when true, existing clone path when false

## 8. Tests

- [x] 8.1 Write unit tests for `detect_default_branch` using a local bare test repo
- [x] 8.2 Write integration test that runs `git-clown --bare` against a local bare repo and asserts: `.bare` directory exists, `.git` file content, worktree directory exists, `.code-workspace` file content
- [x] 8.3 Verify `cargo test` passes and `cargo clippy` reports no warnings

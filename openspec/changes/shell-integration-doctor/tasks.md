## 1. Shell integration

- [x] 1.1 Add `shell/git-clown.sh` with the `git-clown` and `git` wrapper functions (bash/zsh)
- [x] 1.2 Add `git-clown init <shell>` to print the script
- [x] 1.3 Write the repository directory to `$GIT_CLOWN_CD_FILE` after a successful clone, existing clone or bare clone
- [x] 1.4 Make `run_bare_clone` return the worktree path; merge the duplicated branches in `run()`

## 2. Doctor

- [x] 2.1 Checks: git, git-clown on PATH, workspace root, shell detection, current session
- [x] 2.2 rc file state (missing / current / outdated / manual) and end-to-end self-test via hidden `__selftest-cd`
- [x] 2.3 `--install` with an idempotent marked block, and a message naming the rc file and how to reload it
- [x] 2.4 Interactive `[y/N]` offer to install (crossterm), only on a TTY

## 3. Docs and tests

- [x] 3.1 README: shell integration and doctor sections; CLI help text
- [x] 3.2 Unit tests for rc-block handling and report rendering
- [x] 3.3 Integration tests running real bash/zsh in `tests/shell_integration.rs`

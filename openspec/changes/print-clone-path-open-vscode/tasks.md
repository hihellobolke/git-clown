## 1. Clone Result Path

- [ ] 1.1 Define a `CloneResult` struct (or tuple) holding `worktree: PathBuf` and `vscode_target: PathBuf`; use it as the return value of `run_bare_clone`
- [ ] 1.2 Change `run_bare_clone` to return `Result<CloneResult, GitClownError>` — `worktree` is the branch dir, `vscode_target` is the `.code-workspace` file path
- [ ] 1.3 In `run()`, capture the standard clone result path: `dir_path.join(&repo)`; its `vscode_target` is the same path
- [ ] 1.4 In `run()`, capture the already-exists result path: `dir_path.join(&repo)`
- [ ] 1.5 For standard clone: print `Cloned to: <path>` to stdout
- [ ] 1.6 For bare clone: print `Worktree:  <worktree-path>` and `Workspace: <workspace-file-path>` to stdout

## 2. VS Code Open Prompt

- [ ] 2.1 In `prompt_open_vscode`, check `std::io::stdin().is_terminal()` first; return immediately (no output) if stdin is not a TTY
- [ ] 2.2 Add async function `prompt_open_vscode(vscode_target: &Path)` that prints the prompt line and reads one line from stdin with a 5-second `tokio::time::timeout`
- [ ] 2.3 In `prompt_open_vscode`, if input is empty (Enter) or `y`/`Y`, spawn `code <vscode_target>` using `tokio::process::Command`; log a `warn!` if spawn fails
- [ ] 2.4 In `prompt_open_vscode`, if input is `n`/`N` print `(skipped)` and return
- [ ] 2.5 In `prompt_open_vscode`, if `tokio::time::timeout` fires print `(timed out, skipping)` and return
- [ ] 2.6 Call `prompt_open_vscode(&result_path).await` from `run()` after the `Cloned to:` print, for both standard and bare paths

## 3. Tests

- [ ] 3.1 Unit-test `run_bare_clone` returns the correct worktree `PathBuf`
- [ ] 3.2 Verify `cargo test` passes and `cargo clippy` reports no new warnings

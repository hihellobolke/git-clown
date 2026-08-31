## ADDED Requirements

### Requirement: VS Code open prompt after successful clone
After printing the result path, the tool SHALL display an interactive prompt asking the user whether to open the result directory in VS Code. The prompt SHALL read:

```
Open in VS Code? [Y/n] (auto-closing in 5 s):
```

The tool SHALL wait up to 5 seconds for user input.

- If the user presses Enter or types `y` or `Y` (followed by Enter) within 5 seconds:
  - For a **standard clone**: the tool SHALL invoke `code <clone-directory>`.
  - For a **bare clone**: the tool SHALL invoke `code <path-to-.code-workspace-file>` (not the worktree directory), so VS Code opens the full multi-worktree workspace.
- If the user types `n` or `N` (followed by Enter) within 5 seconds, the tool SHALL skip the `code` invocation and print `(skipped)`.
- If 5 seconds elapse with no input, the tool SHALL skip the `code` invocation and print `(timed out, skipping)`.
- The `code` invocation failure (e.g. VS Code not installed) SHALL NOT cause `git-clown` to exit with a non-zero status; a warning SHALL be logged instead.

#### Scenario: User presses Enter to open VS Code after standard clone
- **WHEN** a standard clone has completed and the user presses Enter within 5 seconds
- **THEN** `code <clone-directory>` SHALL be invoked and the tool exits 0

#### Scenario: User presses Enter to open VS Code after bare clone
- **WHEN** a bare clone has completed and the user presses Enter within 5 seconds
- **THEN** `code <path-to-.code-workspace-file>` SHALL be invoked and the tool exits 0

#### Scenario: User types Y to open VS Code
- **WHEN** the user types `y` or `Y` followed by Enter within 5 seconds
- **THEN** the appropriate `code` target (directory or workspace file) SHALL be invoked and the tool exits 0

#### Scenario: User types N to skip
- **WHEN** the user types `n` or `N` followed by Enter within 5 seconds
- **THEN** `code` SHALL NOT be invoked, `(skipped)` is printed, and the tool exits 0

#### Scenario: Prompt times out
- **WHEN** no input is received within 5 seconds
- **THEN** `code` SHALL NOT be invoked, `(timed out, skipping)` is printed, and the tool exits 0

#### Scenario: code CLI not available
- **WHEN** the user confirms and `code` is not on PATH
- **THEN** a warning SHALL be logged, no error is shown to the user, and the tool exits 0

#### Scenario: Non-interactive stdin skips prompt entirely
- **WHEN** stdin is not a TTY (e.g. CI, shell script, piped input)
- **THEN** the prompt SHALL NOT be displayed, `code` SHALL NOT be invoked, and the tool exits 0 immediately after printing the `Cloned to:` line

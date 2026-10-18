## ADDED Requirements

### Requirement: Doctor reports setup problems
`git-clown doctor` SHALL check that git is available, that `git-clown` is on PATH, the workspace root, the user's shell (from `$SHELL` or `--shell`), whether the integration is in the shell's rc file, and whether it is loaded in the current session. It SHALL also run a self-test that loads the integration in a clean shell and checks that both `git-clown` and `git clown` change directory. It SHALL exit non-zero when any check fails.

#### Scenario: Integration missing
- **GIVEN** the rc file does not contain the integration
- **WHEN** the user runs `git-clown doctor` without a terminal
- **THEN** the report marks the rc file as failed, shows the install command, and exits with status 1

#### Scenario: Interactive offer to install
- **GIVEN** the rc file does not contain the integration and stdin/stdout are terminals
- **WHEN** the user runs `git-clown doctor` and presses `y`
- **THEN** the integration is installed, and doctor names the rc file and how to reload it

### Requirement: Idempotent install
`git-clown doctor --install` SHALL add a block delimited by `# >>> git-clown shell integration >>>` and `# <<< git-clown shell integration <<<` to the rc file (`$ZDOTDIR/.zshrc` or `~/.zshrc`; `~/.bashrc`, or `~/.bash_profile` on macOS). It SHALL say which file changed and how to reload it.

#### Scenario: Re-running install
- **GIVEN** the block is already present and current
- **WHEN** the user runs `git-clown doctor --install` again
- **THEN** the rc file is not modified and doctor says nothing changed

#### Scenario: Outdated block
- **GIVEN** the rc file contains an older version of the block
- **WHEN** the user runs `git-clown doctor --install`
- **THEN** only the block is replaced, and the rest of the file is kept

#### Scenario: Commented-out setup line
- **GIVEN** the rc file only mentions `git-clown init <shell>` in a comment, whether on its own line or after an operator (`true;# eval "$(git-clown init zsh)"`)
- **WHEN** the user runs `git-clown doctor --install`
- **THEN** the rc file counts as not installed and the block is added

#### Scenario: Setup done by hand
- **GIVEN** the user loads `git-clown init <shell>` themselves and there is no managed block
- **WHEN** the user runs `git clown doctor` from a shell where the wrapper is loaded
- **THEN** doctor sees the wrapper, reports the integration as set up and offers no install
- **AND** lines outside the managed block are never parsed, so `--install` adds the block regardless (loading the functions twice is harmless)

### Requirement: rc file syntax check
Doctor SHALL have the shell parse the rc file without running it (`bash -n` / `zsh -n`) and SHALL report a syntax error as a failure. A syntax error stops the shell from reading the rest of the file, including the block.

#### Scenario: Unterminated command substitution
- **GIVEN** the rc file contains `value="$(true;# eval "$(git-clown init bash)" )"`, where the comment also swallows the closing `)"`
- **WHEN** the user runs `git-clown doctor --install`
- **THEN** the block is added, the rc syntax check fails with the shell's error message, and doctor exits with status 1

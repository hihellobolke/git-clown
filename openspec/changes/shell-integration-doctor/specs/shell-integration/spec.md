## ADDED Requirements

### Requirement: Shell integration script
`git-clown init <shell>` SHALL print, for `bash` and `zsh`, shell functions named `git-clown` and `git`. The `git` function SHALL pass `git clown ...` to the `git-clown` function, and every other command to the real `git` unchanged.

#### Scenario: Non-clown git commands pass through
- **GIVEN** the integration is loaded
- **WHEN** the user runs `git status`
- **THEN** the real `git status` runs with the same arguments and exit code

### Requirement: Directory hand-off via GIT_CLOWN_CD_FILE
When `GIT_CLOWN_CD_FILE` is set and a clone succeeds, git-clown SHALL write the absolute repository directory to that file. stdout, stderr and the exit code SHALL be unaffected.

#### Scenario: Standard clone changes directory
- **GIVEN** the integration is loaded
- **WHEN** `git clown https://github.com/octocat/Hello-World.git` succeeds
- **THEN** the shell's working directory is `$WORKSPACE_ROOT/github.com/octocat/Hello-World`

#### Scenario: Nested namespace
- **GIVEN** the integration is loaded
- **WHEN** `git clown https://gitlab.com/org/subgroup/repo.git` succeeds
- **THEN** the shell's working directory is the directory git cloned into, `$WORKSPACE_ROOT/gitlab.com/org/repo`

#### Scenario: Path containing a newline
- **GIVEN** `$WORKSPACE_ROOT` contains a newline
- **WHEN** a clone through the wrapper succeeds
- **THEN** the shell changes into the full path

#### Scenario: Strict shell options
- **GIVEN** the integration is loaded in a shell with `set -u`
- **WHEN** the user runs `git` with no arguments
- **THEN** the real `git` runs and the shell keeps going

#### Scenario: Already cloned repository
- **GIVEN** the repository already exists in the workspace
- **WHEN** the user runs `git clown <url>` again
- **THEN** the shell changes into the existing repository directory

#### Scenario: Bare clone changes into the worktree
- **GIVEN** the integration is loaded and the default branch is `main`
- **WHEN** `git clown --bare <url>` succeeds
- **THEN** the shell's working directory is `$WORKSPACE_ROOT/<host>/<org>/<repo>/main`

#### Scenario: Failed clone
- **GIVEN** the integration is loaded
- **WHEN** the clone fails
- **THEN** the working directory is unchanged and the wrapper returns git-clown's exit code

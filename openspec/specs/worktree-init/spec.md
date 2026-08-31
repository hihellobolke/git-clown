# worktree-init Specification

## Purpose
TBD - created by archiving change bare-repo-clone-worktree. Update Purpose after archive.
## Requirements
### Requirement: Default branch worktree is added after bare clone
After a successful bare clone, the tool SHALL detect the repository's default branch and add it as a git worktree at `$WORKSPACE_ROOT/<host>/<org>/<repo>/<branch>/`.

#### Scenario: Default branch is main
- **WHEN** the remote HEAD resolves to `refs/heads/main`
- **THEN** `git worktree add main` SHALL be run from the repo root directory, producing `$WORKSPACE_ROOT/<host>/<org>/<repo>/main/`

#### Scenario: Default branch is master
- **WHEN** the remote HEAD resolves to `refs/heads/master` (and `main` does not exist)
- **THEN** `git worktree add master` SHALL be run from the repo root directory, producing `$WORKSPACE_ROOT/<host>/<org>/<repo>/master/`

#### Scenario: Default branch detection fallback
- **WHEN** `git symbolic-ref HEAD` fails to resolve a branch name
- **THEN** the tool SHALL exit with a non-zero exit code and emit a clear error message indicating the default branch could not be determined

#### Scenario: Worktree already exists
- **WHEN** the initial worktree directory already exists for the detected default branch
- **THEN** the tool SHALL skip the `git worktree add` step and emit an informational message

#### Scenario: Worktree add command runs from repo root
- **WHEN** `git worktree add <branch>` is executed
- **THEN** it SHALL be invoked with the working directory set to `$WORKSPACE_ROOT/<host>/<org>/<repo>/`, not from inside `.bare`


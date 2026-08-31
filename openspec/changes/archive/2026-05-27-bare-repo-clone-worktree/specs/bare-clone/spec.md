## ADDED Requirements

### Requirement: Bare clone via --bare flag
The CLI SHALL accept an optional `--bare` flag. When `--bare` is provided, the repository SHALL be cloned as a bare repository into `$WORKSPACE_ROOT/<host>/<org>/<repo>/.bare` instead of performing a standard clone.

#### Scenario: Bare clone creates .bare directory
- **WHEN** the user runs `git-clown --bare git@github.com:octocat/Hello-World.git`
- **THEN** the directory `$WORKSPACE_ROOT/github.com/octocat/Hello-World/.bare` SHALL exist and be a valid bare git repository

#### Scenario: Bare clone creates .git pointer file
- **WHEN** a bare clone completes successfully
- **THEN** a file at `$WORKSPACE_ROOT/github.com/octocat/Hello-World/.git` SHALL exist with the content `gitdir: ./.bare`

#### Scenario: Bare clone fixes fetch refspec
- **WHEN** a bare clone completes successfully
- **THEN** the git config key `remote.origin.fetch` in `.bare/config` SHALL equal `+refs/heads/*:refs/remotes/origin/*`

#### Scenario: Bare clone runs git fetch
- **WHEN** the refspec is set
- **THEN** `git fetch` SHALL be run so that `refs/remotes/origin/*` are populated

#### Scenario: .bare already exists
- **WHEN** the user runs `git-clown --bare` for a URL whose `.bare` directory already exists
- **THEN** the tool SHALL skip the clone step and emit an informational message, matching the existing behaviour for non-bare re-clones

#### Scenario: Env file is loaded before bare clone
- **WHEN** an `.envrc` or `.env` file is found in the directory hierarchy
- **THEN** its variables SHALL be loaded into the environment before `git clone --bare` is executed

## ADDED Requirements

### Requirement: Unified Git Subprocess Environment
The system MUST execute every git subprocess invoked by git-clown with the same merged runtime environment, composed of inherited process variables and sourced environment-file variables.

#### Scenario: Bare workflow command receives merged environment
- **WHEN** a user runs git-clown with bare workflow enabled and a valid env file is discovered
- **THEN** each bare-workflow git subprocess uses the merged environment map for execution

#### Scenario: Standard clone command receives merged environment
- **WHEN** a user runs git-clown in non-bare mode
- **THEN** the git clone subprocess uses the merged environment map for execution

### Requirement: Environment Propagation Across Bare Workflow Steps
The system SHALL apply merged environment variables to all git subprocess steps in the bare workflow, including clone, config, fetch, default-branch detection, and worktree creation.

#### Scenario: git fetch uses sourced environment variables
- **WHEN** sourced environment variables are present and the bare workflow runs fetch
- **THEN** the git fetch subprocess executes with those sourced variables available

#### Scenario: symbolic-ref and worktree commands use sourced variables
- **WHEN** the bare workflow detects the default branch and creates the initial worktree
- **THEN** both git symbolic-ref and git worktree add subprocesses execute with sourced variables available

### Requirement: Environment Variable Precedence Preservation
The system MUST preserve precedence where environment-file variables override inherited process environment values for all git subprocesses.

#### Scenario: File value overrides inherited value
- **WHEN** the same variable exists in both inherited process env and sourced file env
- **THEN** each git subprocess observes the sourced file value

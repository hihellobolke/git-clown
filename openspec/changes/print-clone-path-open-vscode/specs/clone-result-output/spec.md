## ADDED Requirements

### Requirement: Clone result path is printed to stdout
After a successful git-clown operation the tool SHALL print the result location(s) to stdout.

For a **standard clone** one line is printed:
```
Cloned to: $WORKSPACE_ROOT/<host>/<org>/<repo>
```

For a **bare clone** two lines are printed:
```
Worktree:  $WORKSPACE_ROOT/<host>/<org>/<repo>/<branch>
Workspace: $WORKSPACE_ROOT/<host>/<org>/<repo>/<repo>.code-workspace
```

#### Scenario: Standard clone prints repo directory
- **WHEN** `git-clown git@github.com:octocat/Hello-World.git` completes successfully
- **THEN** exactly one line `Cloned to: <absolute-path>/github.com/octocat/Hello-World` SHALL be written to stdout

#### Scenario: Bare clone prints worktree and workspace paths
- **WHEN** `git-clown --bare git@github.com:octocat/Hello-World.git` completes successfully and the default branch is `main`
- **THEN** a line `Worktree:  <absolute-path>/github.com/octocat/Hello-World/main` SHALL be written to stdout
- **THEN** a line `Workspace: <absolute-path>/github.com/octocat/Hello-World/Hello-World.code-workspace` SHALL be written to stdout

#### Scenario: Failed clone prints no result path
- **WHEN** the git clone command exits with a non-zero status
- **THEN** no `Cloned to:` line SHALL appear on stdout

#### Scenario: Already-cloned repo still prints result path
- **WHEN** `git-clown` detects the repository already exists and skips cloning
- **THEN** the `Cloned to:` line SHALL still be printed with the existing directory path

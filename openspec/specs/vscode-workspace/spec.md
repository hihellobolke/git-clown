# vscode-workspace Specification

## Purpose
TBD - created by archiving change bare-repo-clone-worktree. Update Purpose after archive.
## Requirements
### Requirement: VS Code workspace file is created after bare clone
After a successful bare clone and worktree setup, the tool SHALL create a VS Code workspace file at `$WORKSPACE_ROOT/<host>/<org>/<repo>/<repo>.code-workspace`.

#### Scenario: Workspace file is created with correct content
- **WHEN** bare clone and worktree setup complete successfully
- **THEN** the file `<repo>.code-workspace` SHALL exist at the repo root with the following JSON content:
  ```json
  {
    "folders": [
      {
        "path": "."
      }
    ],
    "settings": {}
  }
  ```

#### Scenario: Workspace file already exists
- **WHEN** `<repo>.code-workspace` already exists at the repo root
- **THEN** the tool SHALL skip writing the file and emit an informational message (no overwrite)

#### Scenario: Workspace file path uses repository name
- **WHEN** cloning `git@github.com:octocat/Hello-World.git` with `--bare`
- **THEN** the workspace file SHALL be named `Hello-World.code-workspace` (using the repo name, not the org or host)


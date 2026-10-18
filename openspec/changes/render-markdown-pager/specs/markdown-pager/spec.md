## ADDED Requirements

### Requirement: README is shown after cloning when requested
When `--readme` is passed, or `GIT_CLOWN_README` is set to a true value, git-clown SHALL show the checkout's markdown README after a successful run.

#### Scenario: Standard clone with --readme
- **GIVEN** stdout is a terminal
- **WHEN** `git-clown --readme https://github.com/octocat/Spoon-Knife.git` clones successfully
- **THEN** the repository's `README.md` SHALL open in the full-screen pager
- **THEN** after the user quits the pager, git-clown SHALL exit with the clone's status

#### Scenario: Bare clone with --readme
- **GIVEN** the default branch is `main`
- **WHEN** `git-clown --bare --readme <url>` completes successfully
- **THEN** the README of the `<repo>/main` worktree SHALL be shown

#### Scenario: Repository already cloned
- **GIVEN** the repository already exists in the workspace
- **WHEN** `git-clown --readme <url>` runs
- **THEN** the existing checkout's README SHALL be shown

#### Scenario: Failed clone
- **WHEN** git clone exits with a non-zero status
- **THEN** no README SHALL be shown and git-clown SHALL exit with that status

#### Scenario: No markdown README
- **GIVEN** the checkout has none of `README.md`, `README.markdown`, `readme.md`, `Readme.md`
- **WHEN** `git-clown --readme <url>` completes
- **THEN** a warning SHALL be logged and the exit status SHALL be the clone's

#### Scenario: Setting turns the README on or off
- **GIVEN** `GIT_CLOWN_README=1` (or `true`, `yes`, `on`)
- **WHEN** `git-clown <url>` completes successfully
- **THEN** the README SHALL be shown as if `--readme` were passed
- **GIVEN** `GIT_CLOWN_README=0` (or `false`, `no`, `off`)
- **THEN** no README SHALL be shown

### Requirement: git-clown's own docs are viewable
`git-clown --docs` SHALL show git-clown's README, embedded in the binary, and exit with status 0 without needing a git URL.

#### Scenario: Docs without a URL
- **WHEN** `git-clown --docs` runs
- **THEN** the documentation SHALL be shown and no clone SHALL happen

#### Scenario: Docs with clone options
- **WHEN** `git-clown --docs <url>` or `git-clown --docs --bare` runs
- **THEN** git-clown SHALL fail with a usage error

### Requirement: Pager renders markdown and is keyboard driven
The pager SHALL render headings, emphasis, inline code, code blocks, lists, task lists, block quotes, links, images, tables and rules, wrapped to the terminal width, with a status bar showing the title, position and keys.

#### Scenario: Navigation
- **GIVEN** the pager is open
- **WHEN** the user presses `j`/`↓`/`Enter`, `k`/`↑`, `Space`/`f`/`PageDown`, `b`/`PageUp`, `d`, `u`, `g`/`Home`, `G`/`End`
- **THEN** the view SHALL scroll by a line, a page, half a page, or to the top or end, never past either end

#### Scenario: Quit
- **WHEN** the user presses `q`, `Esc` or `Ctrl-C`
- **THEN** the pager SHALL close and the terminal SHALL be restored

#### Scenario: Resize
- **WHEN** the terminal is resized
- **THEN** the document SHALL be re-wrapped to the new width

#### Scenario: Untrusted content
- **GIVEN** a README containing escape or other control characters
- **WHEN** it is rendered
- **THEN** the control characters SHALL NOT be written to the terminal

### Requirement: Plain output when stdout is not a terminal
When stdout is not a terminal, or `TERM` is `dumb`, the markdown SHALL be written to stdout unchanged instead of opening the pager.

#### Scenario: Piped docs
- **WHEN** `git-clown --docs | less` runs
- **THEN** the README source SHALL be written to the pipe and git-clown SHALL exit with status 0

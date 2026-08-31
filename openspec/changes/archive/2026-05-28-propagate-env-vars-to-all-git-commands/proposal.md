## Why

Environment variables sourced from .envrc or .env are currently applied to some git subprocesses but not all. This creates inconsistent behavior, where operations like fetch or worktree setup can fail in environments that require custom SSH, proxy, auth, or git config variables.

## What Changes

- Introduce a consistent command construction path so every git subprocess invocation receives the same merged environment variables.
- Apply sourced environment variables to all git operations in standard clone and bare-clone workflows, including clone, config, fetch, symbolic-ref, and worktree add.
- Keep current precedence behavior where file-sourced variables override the inherited process environment.
- Add tests that verify non-clone git subprocesses are executed with the propagated environment.

## Capabilities

### New Capabilities
- `git-command-env-propagation`: Ensure all git command invocations performed by git-clown use the merged runtime environment (system env plus sourced file variables).

### Modified Capabilities
- None.

## Impact

- Affected code: git subprocess helper/orchestration logic in [src/main.rs](src/main.rs), especially bare workflow functions.
- APIs: No external CLI flag or argument changes.
- Dependencies: No new external dependencies required; optional internal refactor only.
- Systems/teams: Developers using private hosts, proxies, tokens, SSH command overrides, or custom git env in env files.
- Rollback plan: Revert the env propagation refactor/changes and restore previous per-call command setup.
- Testing plan and scenarios:
- Validate clone and bare workflows still succeed with no env file.
- Validate sourced env vars are passed to clone, fetch, symbolic-ref, and worktree commands.
- Validate precedence remains sourced-file-over-process-env.

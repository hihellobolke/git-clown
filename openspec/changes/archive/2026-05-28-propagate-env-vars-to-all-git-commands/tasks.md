## 1. Command Construction Refactor

- [x] 1.1 Add a shared helper for constructing git subprocess commands with current directory, args, and merged environment application
- [x] 1.2 Replace duplicate per-function env injection logic with the shared helper where possible

## 2. Bare Workflow Env Propagation

- [x] 2.1 Update bare-workflow function signatures to accept merged environment references where missing
- [x] 2.2 Apply merged environment to git config and git fetch subprocess invocations in refspec/fetch flow
- [x] 2.3 Apply merged environment to default-branch detection and worktree creation subprocess invocations

## 3. Validation and Tests

- [x] 3.1 Add or update tests to verify env propagation for non-clone git subprocesses in bare workflow
- [x] 3.2 Verify standard clone and bare workflows remain successful without env files
- [x] 3.3 Verify sourced env values override inherited process env values during git subprocess execution

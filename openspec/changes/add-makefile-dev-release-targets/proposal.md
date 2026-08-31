## Why

Building and installing the `git-clown` binary currently requires manual commands and path handling. A Makefile with clear `dev` and `release` targets makes local iteration and distribution faster, consistent, and less error-prone.

## What Changes

- Add a project `Makefile` with:
- `TEST_BIN_PATH ?= $(HOME)/bin` default variable for development install destination.
- `RELEASE_BIN_PATH ?= $(HOME)/bin` default variable for release install destination.
- `dev` target that runs `cargo build` and copies `target/debug/git-clown` to `$(TEST_BIN_PATH)/git-clown`, overwriting if it already exists.
- `release` target that runs `cargo build --release` and copies `target/release/git-clown` to `$(RELEASE_BIN_PATH)/git-clown`, overwriting if it already exists.
- Ensure destination directories are created when missing.
- Include `PHONY` declarations and concise help comments for maintainability.

## Capabilities

### New Capabilities
- `makefile-build-install`: Provide standardized Make targets for development and release build/install flows, including overwrite behavior and configurable destination paths.

### Modified Capabilities
- None.

## Impact

- Affected code: New top-level `Makefile` and optional README usage notes.
- APIs: No runtime CLI API change for `git-clown`.
- Dependencies: No new Rust dependencies.
- Systems/teams: Developers using local toolchain and CI jobs that may adopt `make dev`/`make release`.
- Rollback plan: Remove or revert the `Makefile` and any README updates; users can return to direct Cargo commands.
- Testing plan and scenarios:
- Run `make dev` and verify `$(TEST_BIN_PATH)/git-clown` exists and is executable.
- Re-run `make dev` to confirm overwrite behavior.
- Run `make release` and verify `$(RELEASE_BIN_PATH)/git-clown` exists and is executable.
- Validate that non-default paths work: `make dev TEST_BIN_PATH=/tmp/bin`.

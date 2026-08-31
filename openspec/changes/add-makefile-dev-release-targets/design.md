## Context

The project currently documents manual Cargo commands for local development and release builds. There is no standardized command interface for build-plus-install workflows, which leads to repeated manual steps and inconsistent destination paths across developers.

This change introduces a top-level Makefile to standardize local workflows while preserving existing Rust build behavior.

## Goals / Non-Goals

**Goals:**
- Add a `dev` target that builds debug binaries and installs to a configurable destination.
- Add a `release` target that builds release binaries and installs to a configurable destination.
- Ensure install operations overwrite existing binaries and create destination directories when missing.
- Keep behavior simple and deterministic for local developer use.

**Non-Goals:**
- Changing `git-clown` runtime behavior or CLI arguments.
- Adding packaging, signing, or cross-compilation workflows.
- Replacing direct Cargo usage in CI unless a separate change explicitly adopts it.

## Decisions

1. Use a top-level Makefile with explicit targets (`dev`, `release`) and default variables.
- Rationale: Provides discoverable, single-command workflows with optional path overrides.
- Alternative considered: shell scripts (`scripts/dev.sh`, `scripts/release.sh`). Rejected to avoid duplicating target metadata and to keep common developer ergonomics.

2. Use `TEST_BIN_PATH ?= $(HOME)/bin` and `RELEASE_BIN_PATH ?= $(HOME)/bin`.
- Rationale: Supports local defaults while allowing caller override at invocation time.
- Alternative considered: hard-coded path in commands. Rejected because it reduces portability and user control.

3. Copy built binaries with overwrite semantics.
- Rationale: User requirement explicitly expects replacement when destination file exists.
- Alternative considered: fail if destination exists. Rejected because it adds friction during iterative development.

4. Ensure destination directory exists before copy.
- Rationale: Avoids errors in fresh environments and keeps target behavior idempotent.

## Risks / Trade-offs

- [Risk] Local path permissions can cause copy failures.
  - Mitigation: Use user-writable defaults and surface command errors clearly.
- [Risk] Make availability differs across systems.
  - Mitigation: Keep direct Cargo commands documented; Makefile is additive, not mandatory.
- [Trade-off] Two destination variables (`TEST_BIN_PATH`, `RELEASE_BIN_PATH`) increase configuration surface.
  - Mitigation: Keep defaults identical and names explicit to preserve clarity.

## Migration Plan

1. Add Makefile with variables and targets.
2. Validate behavior with local execution:
- `make dev`
- `make release`
- path override examples.
3. Optionally document new usage in README.
4. Rollback by reverting Makefile and documentation changes if problems appear.

## Open Questions

- Should README include a short "Makefile targets" section in this change or a follow-up?
- Should a `help` target be included now, or deferred to keep scope minimal?

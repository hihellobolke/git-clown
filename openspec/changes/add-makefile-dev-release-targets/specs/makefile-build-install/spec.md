## ADDED Requirements

### Requirement: Development Build and Install Target
The system MUST provide a Makefile target named `dev` that SHALL run `cargo build`, ensure the destination directory exists, and copy the debug binary to `$(TEST_BIN_PATH)/git-clown` while overwriting an existing destination file.

#### Scenario: Default dev install path is used
- **WHEN** a developer runs `make dev` without overriding variables
- **THEN** the system builds `target/debug/git-clown` and installs it to `$(HOME)/bin/git-clown`

#### Scenario: Custom dev install path is used
- **WHEN** a developer runs `make dev TEST_BIN_PATH=/tmp/bin`
- **THEN** the system installs the debug binary to `/tmp/bin/git-clown`

#### Scenario: Existing dev binary is replaced
- **WHEN** `$(TEST_BIN_PATH)/git-clown` already exists and a developer runs `make dev`
- **THEN** the existing file is overwritten with the newly built debug binary

### Requirement: Release Build and Install Target
The system MUST provide a Makefile target named `release` that SHALL run `cargo build --release`, ensure the destination directory exists, and copy the release binary to `$(RELEASE_BIN_PATH)/git-clown` while overwriting an existing destination file.

#### Scenario: Default release install path is used
- **WHEN** a developer runs `make release` without overriding variables
- **THEN** the system builds `target/release/git-clown` and installs it to `$(HOME)/bin/git-clown`

#### Scenario: Custom release install path is used
- **WHEN** a developer runs `make release RELEASE_BIN_PATH=/tmp/bin`
- **THEN** the system installs the release binary to `/tmp/bin/git-clown`

#### Scenario: Existing release binary is replaced
- **WHEN** `$(RELEASE_BIN_PATH)/git-clown` already exists and a developer runs `make release`
- **THEN** the existing file is overwritten with the newly built release binary

### Requirement: Configurable Destination Variables
The Makefile MUST define default destination variables that can be overridden at invocation time.

#### Scenario: Default values are available
- **WHEN** the Makefile is loaded with no variable overrides
- **THEN** `TEST_BIN_PATH` defaults to `$(HOME)/bin` and `RELEASE_BIN_PATH` defaults to `$(HOME)/bin`

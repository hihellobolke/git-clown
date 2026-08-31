## 1. Makefile Setup

- [x] 1.1 Add a top-level Makefile with default variables `TEST_BIN_PATH ?= $(HOME)/bin` and `RELEASE_BIN_PATH ?= $(HOME)/bin`
- [x] 1.2 Declare `.PHONY` targets for `dev` and `release`

## 2. Dev Build and Install Target

- [x] 2.1 Implement `dev` target to run `cargo build`
- [x] 2.2 Ensure `$(TEST_BIN_PATH)` exists before install
- [x] 2.3 Copy `target/debug/git-clown` to `$(TEST_BIN_PATH)/git-clown` with overwrite behavior

## 3. Release Build and Install Target

- [x] 3.1 Implement `release` target to run `cargo build --release`
- [x] 3.2 Ensure `$(RELEASE_BIN_PATH)` exists before install
- [x] 3.3 Copy `target/release/git-clown` to `$(RELEASE_BIN_PATH)/git-clown` with overwrite behavior

## 4. Verification and Documentation

- [x] 4.1 Verify `make dev` installs binary at the default destination
- [x] 4.2 Verify `make release` installs binary at the default destination
- [x] 4.3 Verify override paths work (for example `make dev TEST_BIN_PATH=/tmp/bin`)
- [x] 4.4 Update README with `make dev` and `make release` usage if needed

TEST_BIN_PATH ?= $(HOME)/bin
RELEASE_BIN_PATH ?= $(HOME)/bin

.PHONY: dev release

dev:
	cargo build
	mkdir -p $(TEST_BIN_PATH)
	cp -f target/debug/git-clown $(TEST_BIN_PATH)/git-clown

release:
	cargo build --release
	mkdir -p $(RELEASE_BIN_PATH)
	cp -f target/release/git-clown $(RELEASE_BIN_PATH)/git-clown

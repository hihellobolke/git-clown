# Git Clown

Git Clown is a Rust-based command-line tool that extends git clone functionality by automatically organizing your repositories into a structured workspace based on their URLs.

## Introduction

When working with multiple repositories, maintaining a clean and organized workspace structure becomes challenging. Git Clown solves this problem by automatically creating a structured directory hierarchy based on the repository URL. For example, a repository at `https://github.com/example/repo.git` would be cloned to `$WORKSPACE_ROOT/github.com/example/repo`.

Key features:
- Automatic directory structure creation based on repository URL
- Environment variable loading from `.envrc` or `.env` files
- Support for both HTTPS and SSH Git URLs
- Colored logging output for enhanced readability
- Async operations for better performance
- Optional shell integration (bash, zsh) so `git clown <url>` leaves you inside the cloned repository
- `git-clown doctor` to check your setup and install the shell integration

## Installation

### Prerequisites
- Rust and Cargo (1.56.0 or later)
- Git

### Building from Source

1. Clone the repository:
```sh
git clone https://github.com/your-org/git-clown.git
cd git-clown
```

2. Build the project:
```sh
cargo build --release
```

Or use the Makefile targets to build and install:
```sh
make dev
make release
```

3. Add the binary to your PATH (git runs `git clown` by finding `git-clown` on your PATH):
```sh
# Option 1: Copy to a directory in your PATH
cp target/release/git-clown ~/.local/bin/
# or
# Option 2: Create a symbolic link
ln -s $(pwd)/target/release/git-clown ~/.local/bin/
```

4. Optionally enable the shell integration, so `git clown` also changes into the cloned directory:
```sh
git-clown doctor --install
```

Makefile install destinations:
- `make dev` builds debug and copies to `$(HOME)/bin/git-clown` (override with `TEST_BIN_PATH=/path`)
- `make release` builds release and copies to `$(HOME)/bin/git-clown` (override with `RELEASE_BIN_PATH=/path`)

## Usage

### Basic Usage

```sh
git-clown https://github.com/example/repo.git
```

This will:
1. Create the directory `$WORKSPACE_ROOT/github.com/example/`
2. Check if the repository is already cloned (if it is, it will skip cloning and just show a message)
3. Find a `.envrc` or `.env` file starting from this directory and working up to `$WORKSPACE_ROOT`
4. Load environment variables from the found file
5. Clone the repository into this directory (if not already cloned)
6. With the [shell integration](#shell-integration-cd-into-the-clone) loaded, `cd` into the repository

### Shell Integration: cd into the clone

A program cannot change the working directory of the shell that started it, so
on its own `git-clown` can only tell you where the repository is. With the shell
integration loaded, `git clown <url>` (and `git-clown <url>`) also `cd`s into
the cloned repository. For `--bare` clones it `cd`s into the default branch
worktree. If the repository was already cloned, it `cd`s into the existing one.

Install it with:

```sh
git-clown doctor --install            # for the shell in $SHELL
git-clown doctor --install --shell bash --shell zsh
```

This adds a small marked block to your rc file (`~/.zshrc`, or
`$ZDOTDIR/.zshrc`; `~/.bashrc` for bash, `~/.bash_profile` on macOS). Running it
again changes nothing, and an outdated block is updated in place. Reload your
shell afterwards with `source ~/.zshrc` (or `~/.bashrc`), or open a new terminal.

To set it up by hand, add this line to your rc file instead:

```sh
eval "$(command git-clown init zsh)"   # or: init bash
```

Doctor only recognises the marked block in rc files. If you set it up by hand,
run `git clown doctor` from a shell that has it loaded: doctor then sees the
wrapper and counts the integration as set up.

How it works: `git-clown init <shell>` prints two shell functions. `git-clown`
runs the real binary with `GIT_CLOWN_CD_FILE` pointing at a temp file. After a
successful clone the binary writes the repository path into that file, and the
function `cd`s there. `git` sends `git clown ...` to that function and passes
every other command straight to the real `git`. stdout, stderr and exit codes
are unchanged, and nothing happens when the clone fails.

> Note: the integration defines a `git` shell function. If you already have your
> own `git` function, whichever is loaded last wins. In that case, have your
> function call `git-clown "$@"` (after `shift`) when its first argument is `clown`.

### Doctor

```sh
git-clown doctor
```

Checks your setup and reports what is missing:

- `git` is installed, and `git-clown` is on `PATH` (so `git clown` works)
- the workspace root (`$WORKSPACE_ROOT` or `~/ws`)
- your shell (from `$SHELL`), and whether the integration is loaded in the
  current session
- whether the integration block is in your rc file, and whether your shell can
  parse that file (`bash -n` / `zsh -n`, nothing is run). A syntax error stops
  the shell from reading the rest of the file, including the block.
- a self-test that loads the integration in a clean shell and checks that
  `git-clown` and `git clown` really `cd`

If the integration is missing and you run doctor in a terminal, it asks whether
to install it. Use `--install` to install without asking, and `--shell` to pick
the shell(s). It exits with a non-zero status when a check fails.

### Environment Variables

- `WORKSPACE_ROOT`: Set this environment variable to specify your workspace root directory. If not set, it defaults to `~/ws/`.
- `GIT_CLOWN_CD_FILE`: Set by the shell integration. After a successful clone, git-clown writes the repository directory to this file.

### Environment File Support

Git Clown supports loading environment variables from:
- `.envrc` files (with `export KEY=VALUE` syntax)
- `.env` files (with `KEY=VALUE` syntax)

These files are searched for starting from the target directory and traversing up to the workspace root. The environment variables are then passed to the git clone command.

## Contributing

Contributions are welcome! Here's how you can contribute:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-new-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Documentation

To generate and view the documentation:

```sh
cargo doc --open
```

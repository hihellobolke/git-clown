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
- Optional terminal pager that renders the cloned repository's README (and these docs) as formatted markdown

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

3. Add the binary to your PATH:
```sh
# Option 1: Copy to a directory in your PATH
cp target/release/git-clown ~/.local/bin/
# or
# Option 2: Create a symbolic link
ln -s $(pwd)/target/release/git-clown ~/.local/bin/
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

### Reading the README in a Pager

Add `--readme` to read the repository's README right after cloning:

```sh
git-clown --readme https://github.com/example/repo.git
```

Once the clone succeeds (or the repository turns out to be cloned already), the README opens in a full-screen pager with headings, emphasis, lists, code blocks, tables and links formatted for the terminal. Text is wrapped to the terminal width, table cells included; only a table with more columns than the screen can hold is wider, and then you can scroll sideways. With `--bare`, the README of the default branch worktree is shown. If the repository has no `README.md` (or `README.markdown`), a warning is logged and nothing else happens. A failed clone never opens the pager.

To read git-clown's own documentation (this file) the same way:

```sh
git-clown --docs
```

Keys in the pager:

| Key | Action |
|-----|--------|
| `q`, `Esc`, `Ctrl-C` | Quit |
| `↓`, `j`, `Enter` / `↑`, `k` | Scroll one line down / up |
| `Space`, `f`, `PageDown` / `b`, `PageUp` | Scroll one page down / up |
| `d` / `u` | Scroll half a page down / up |
| `g`, `Home` / `G`, `End` | Go to the top / end |
| `←`, `h` / `→`, `l` | Scroll sideways, when the status bar says there is more to the side |

The pager is only used when stdout is a terminal. When the output is piped or redirected (or `TERM=dumb`), the markdown is printed unchanged instead, so `git-clown --docs | less` or `git-clown --docs > git-clown.md` work as expected.

To always show the README, set `GIT_CLOWN_README` in your shell profile instead of passing the flag each time:

```sh
export GIT_CLOWN_README=1
```

### Environment Variables

- `WORKSPACE_ROOT`: Set this environment variable to specify your workspace root directory. If not set, it defaults to `~/ws/`.
- `GIT_CLOWN_README`: Set to `1` (or `true`, `yes`, `on`) to show the README after every clone, as if `--readme` were passed. `0`, `false`, `no` or `off` turn it off.

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

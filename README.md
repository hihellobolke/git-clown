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

3. Add the binary to your PATH:
```sh
# Option 1: Copy to a directory in your PATH
cp target/release/git-clown ~/.local/bin/
# or
# Option 2: Create a symbolic link
ln -s $(pwd)/target/release/git-clown ~/.local/bin/
```

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

### Environment Variables

- `WORKSPACE_ROOT`: Set this environment variable to specify your workspace root directory. If not set, it defaults to `~/ws/`.

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

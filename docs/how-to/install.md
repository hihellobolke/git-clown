# Install git-clown

This guide shows how to build git-clown from source and put it where both your
shell and git can find it.

You need git and a Rust toolchain (`cargo`, version 1.88 or later; some of the
locked dependencies, such as ratatui, need 1.88). Check yours with
`cargo --version`, and update it with `rustup update stable` if it is older.

## Build and install with make

From a checkout of the git-clown repository:

```sh
make release
```

This builds an optimised binary and copies it to `~/bin/git-clown`. To install
somewhere else, pass the directory:

```sh
make release RELEASE_BIN_PATH="$HOME/.local/bin"
```

For a debug build (faster to compile, useful while working on git-clown
itself), use `make dev`. It installs to `~/bin` too; change that with
`TEST_BIN_PATH`:

```sh
make dev TEST_BIN_PATH="$HOME/.local/bin"
```

## Build and install by hand

Pick a directory for the binary, such as `~/.local/bin`, and create it if it
doesn't exist:

```sh
mkdir -p ~/.local/bin
cargo build --release
cp target/release/git-clown ~/.local/bin/
```

Or link the build output, so that every later `cargo build --release` updates
the installed command:

```sh
ln -s "$(pwd)/target/release/git-clown" ~/.local/bin/
```

## Make sure the directory is on PATH

git runs `git clown` by looking for an executable called `git-clown` on your
`PATH`, so the directory you installed into must be on it. Check with:

```sh
command -v git-clown
```

If that prints nothing, add the directory you installed into to `PATH` in your
shell's rc file (`~/.zshrc`, `~/.bashrc`, or `~/.bash_profile` on macOS).

If you ran `make release` or `make dev` without a path, the binary is in
`~/bin`:

```sh
export PATH="$HOME/bin:$PATH"
```

If you installed into `~/.local/bin` (with `RELEASE_BIN_PATH` or
`TEST_BIN_PATH`, or by hand as above), add that directory instead:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

For any other directory, use it in place of `$HOME/.local/bin`. Then open a new
terminal (or `source` the rc file) and run `command -v git-clown` again: it
should print the path of the binary you installed.

## Check the installation

```sh
git-clown --version
git-clown doctor
```

In the doctor's **Environment** section, `git` and `git-clown on PATH` should
both have a `✓`. If `git-clown on PATH` shows `!` and says the doctor is
running a different binary, you have more than one copy installed: remove the
old one, or put the directory of the one you want first on `PATH`.

## Next steps

To have `git clown <url>` change into the cloned repository, set up the
[shell integration](set-up-shell-integration.md).

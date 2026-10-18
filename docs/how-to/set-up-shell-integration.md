# Set up the shell integration

With the shell integration loaded, `git clown <url>` and `git-clown <url>`
change your shell's directory to the repository after a successful clone. This
guide covers installing, checking, updating and removing it. bash and zsh are
supported.

## Install it into your rc file

```sh
git-clown doctor --install
```

This installs the integration for the shell in `$SHELL`. To choose the shell,
or install for both, use `--shell` (repeat it for more than one):

```sh
git-clown doctor --install --shell bash --shell zsh
```

The doctor adds a marked block to the rc file and says which file it changed:

| Shell | File |
|-------|------|
| zsh | `$ZDOTDIR/.zshrc`, or `~/.zshrc` when `ZDOTDIR` is not set |
| bash on Linux | `~/.bashrc` |
| bash on macOS | `~/.bash_profile` |

Then load it into the shell you are using, or open a new terminal:

```sh
source ~/.zshrc      # or ~/.bashrc, ~/.bash_profile
```

Running `--install` again changes nothing.

## Install it from the doctor's prompt

If you run `git-clown doctor` in a terminal and the integration is missing (or
outdated), the doctor asks:

```text
Install the zsh integration in ~/.zshrc? [y/N]
```

Press `y` to install it. Any other key leaves the file alone.

## Set it up by hand

If you manage your rc file yourself (for example with a dotfiles framework),
add this line instead of the managed block:

```sh
eval "$(command git-clown init zsh)"    # or: init bash
```

Put it after the line that adds git-clown's directory to `PATH`.

The doctor only recognises its own marked block in rc files, so it reports
`rc file … not installed` for a hand-written line. Run the doctor from a shell
that has the integration loaded: it then sees the integration at work and
reports the rc file as `✓ … loaded in this session`.

## Check that it works

```sh
git clown doctor
```

Look for these lines:

```text
  ✓ this session       integration loaded (zsh); `git clown <url>` will cd into the clone
  ✓ rc file            installed in ~/.zshrc
  ✓ rc syntax          zsh parses it cleanly
  ✓ self-test          `git-clown` and `git clown` wrappers cd correctly
```

If one of them is not a `✓`, see [Fix setup problems with doctor](fix-setup-problems.md).

## Keep your own `git` function

The integration defines a shell function called `git`, which sends
`git clown …` to git-clown and everything else to the real git. If you already
have a `git` function, whichever is loaded last wins.

To keep yours, load the integration first and have your function hand
`clown` over to the `git-clown` function:

```sh
eval "$(command git-clown init zsh)"

function git {
  if [ "${1-}" = "clown" ]; then
    shift
    git-clown "$@"
    return
  fi
  # ... your own handling ...
  command git "$@"
}
```

## Update it after upgrading git-clown

Usually nothing is needed: the block runs `git-clown init` each time a shell
starts, so new shells get the functions from the binary you have installed.

If a new version changes the block itself, the doctor reports
`! rc file  outdated block in …`. Replace it with:

```sh
git-clown doctor --install
```

The old block is replaced in place; the rest of the file is not touched.

## Remove it

Delete the block between these two lines (including the lines themselves) from
your rc file, then open a new terminal:

```text
# >>> git-clown shell integration >>>
...
# <<< git-clown shell integration <<<
```

To remove it from the current shell without opening a new one:

```sh
unset -f git git-clown
```

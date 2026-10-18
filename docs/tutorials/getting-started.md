# Getting started with git-clown

In this tutorial you will install git-clown, clone your first repository into
an organised workspace, set up the shell integration so that `git clown` leaves
you inside the clone, and read a repository's README in the terminal pager.

It takes about ten minutes. At the end you will have a working setup that you
can use every day.

## Before you start

You need:

- a Linux or macOS machine with **bash** or **zsh** as your shell
- **git**
- a **Rust toolchain** (`cargo`), version 1.88 or later. Get it from
  <https://rustup.rs> if you don't have it.
- network access to GitHub

The tutorial uses the default workspace, `~/ws`. If you have already set
`WORKSPACE_ROOT` in your environment, your repositories go there instead, and
every `~/ws` below means that directory.

## Step 1: Build and install git-clown

Get the source and build it:

```sh
git clone https://github.com/hihellobolke/git-clown.git
cd git-clown
make release
```

`make release` builds an optimised binary and copies it to `~/bin/git-clown`.

Check that your shell can find it:

```sh
git-clown --version
```

You should see:

```text
git-clown 0.1.0
```

If you see `command not found` instead, `~/bin` is not on your `PATH`. Add this
line to your `~/.zshrc` (or `~/.bashrc`), open a new terminal and try again:

```sh
export PATH="$HOME/bin:$PATH"
```

## Step 2: Clone your first repository

Go back to your home directory and clone a small example repository. Notice
that you type `git clown`, as if it were a built-in git command:

```sh
cd ~
git clown https://github.com/octocat/Spoon-Knife.git
```

git-clown logs each step as it goes. The output looks like this (your paths
will show your own home directory):

```text
[INFO  git_clown] Git URL: https://github.com/octocat/Spoon-Knife.git
[INFO  git_clown] Successfully parsed HTTPS Git URL: host=github.com, organization=octocat, repository=Spoon-Knife
[INFO  git_clown] Successfully created directory: /home/you/ws/github.com/octocat
[WARN  git_clown] No environment file (.envrc or .env) found in path hierarchy
[WARN  git_clown] No environment file found, proceeding with system environment only
[INFO  git_clown] Executing git clone for https://github.com/octocat/Spoon-Knife.git in /home/you/ws/github.com/octocat
Cloning into 'Spoon-Knife'...

[INFO  git_clown] Git clone completed successfully
[INFO  git_clown] Repository is at /home/you/ws/github.com/octocat/Spoon-Knife
[INFO  git_clown] Tip: run git-clown doctor --install so `git clown` also cd's into the repository
[INFO  git_clown] Git-clown completed with status code: 0
```

The two warnings about environment files are expected; you don't have any yet.

Now look at where the repository went:

```sh
ls ~/ws/github.com/octocat/Spoon-Knife
```

```text
index.html  README.md  styles.css
```

You didn't choose a directory. git-clown built the path from the URL: the host
(`github.com`), then the owner (`octocat`), then the repository
(`Spoon-Knife`).

## Step 3: Run the same command again

Run exactly the same command a second time:

```sh
git clown https://github.com/octocat/Spoon-Knife.git
```

This time git-clown doesn't clone. Among the log lines you'll see:

```text
[INFO  git_clown] Repository Spoon-Knife already exists, skipping git clone
[INFO  git_clown] Repository is at /home/you/ws/github.com/octocat/Spoon-Knife
```

Because every URL has exactly one place in the workspace, git-clown can tell
that the repository is already there. Running it again is always safe.

Notice that your shell is still in your home directory:

```sh
pwd
```

```text
/home/you
```

The tip at the end of the log told you how to change that. That's the next
step.

## Step 4: Install the shell integration

Run the doctor with `--install`:

```sh
git-clown doctor --install
```

The first line of the report tells you what it changed:

```text
✓ Added the zsh integration to ~/.zshrc. Reload it with source ~/.zshrc or open a new terminal.
```

(With bash you will see `~/.bashrc`, or `~/.bash_profile` on macOS.)

Below that, the doctor checks your setup. The line that matters right now is
the self-test:

```text
  ✓ self-test          `git-clown` and `git clown` wrappers cd correctly
```

One line still has a `!`:

```text
  ! this session       integration not loaded in the shell that ran doctor
```

That's because your current shell read its rc file before the doctor changed
it. Reload it, using the file the doctor named:

```sh
source ~/.zshrc
```

## Step 5: Clone and land inside the repository

Go back home and clone again:

```sh
cd ~
git clown https://github.com/octocat/Spoon-Knife.git
pwd
```

```text
/home/you/ws/github.com/octocat/Spoon-Knife
```

This time your shell moved into the repository. From now on, whenever a clone
succeeds (or the repository was already there), `git clown` leaves you inside
it.

Confirm that the doctor agrees:

```sh
git clown doctor
```

The session line now has a tick:

```text
  ✓ this session       integration loaded (zsh); `git clown <url>` will cd into the clone
```

and the report ends with `Everything looks good.`

## Step 6: Read the README without leaving the terminal

Add `--readme` to the same command:

```sh
git clown --readme https://github.com/octocat/Spoon-Knife.git
```

The repository is already cloned, so git-clown goes straight to its README and
opens it full-screen, with headings and emphasis formatted. The bottom line is
a status bar showing the file name, your position and the keys you can use.

Press `j` and `k` to scroll, then press `q` to close the pager. When the pager
closes, your shell is in the repository, as in the previous step.

git-clown's own documentation opens the same way:

```sh
git-clown --docs
```

Press `q` when you're done.

## What you've done

You have:

- installed git-clown and run it as `git clown`
- seen how a URL decides where a repository goes in `~/ws`
- seen that running the same clone again is harmless
- installed the shell integration and checked it with `git-clown doctor`
- read a README in the terminal pager

## Next steps

- To keep your repositories somewhere other than `~/ws`, see
  [Change the workspace root](../how-to/change-workspace-root.md).
- To give git different settings for one host or organisation (another SSH
  key, for example), see
  [Use environment files for git settings](../how-to/use-environment-files.md).
- To work on several branches side by side, see
  [Clone a repository with worktrees](../how-to/clone-with-worktrees.md).
- To understand why the shell integration is needed at all, read
  [How the shell integration works](../explanation/shell-integration.md).
- For every command and option, see the [command reference](../reference/cli.md).

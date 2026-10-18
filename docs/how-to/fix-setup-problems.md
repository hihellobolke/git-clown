# Fix setup problems with doctor

`git-clown doctor` checks your setup and prints one line per check. This guide
tells you what to do about each line that isn't a `✓`. For what every check
looks at, see the [doctor reference](../reference/doctor.md).

Start by running it from the shell you normally use:

```sh
git-clown doctor
```

`✗` lines are problems and make the doctor exit with status 1. `!` lines are
warnings. The indented `↳` line under a check is the doctor's own hint.

## `✗ git  not found on PATH`

Install git (<https://git-scm.com/downloads>) and make sure `git --version`
works in a new terminal.

## `✗ git-clown on PATH  not found`

git can only run `git clown` if an executable named `git-clown` is on your
`PATH`. Install it into a directory on `PATH`, or add its directory to `PATH`.
See [Install git-clown](install.md).

## `! git-clown on PATH  …` with "a different binary"

The `git-clown` that `PATH` finds first is not the one you ran. This usually
means an old copy is still installed. Find all copies:

```sh
which -a git-clown
```

Remove the ones you don't want, or reorder `PATH` so the right one comes
first.

## `✗ workspace root  … Exists but is not a directory`

Something that is not a directory is in the way: a file at `~/ws`, or at the
path in `WORKSPACE_ROOT`. Move it, or point `WORKSPACE_ROOT` somewhere else
(see [Change the workspace root](change-workspace-root.md)).

`! workspace root … Does not exist yet` needs no action: the directory is
created on the first clone.

## `! your shell  … is not supported`

The shell integration works with bash and zsh. If `$SHELL` is another shell
(fish, for example), git-clown still clones, but it can't change your
directory. If you also use bash or zsh, name it:

```sh
git-clown doctor --shell zsh
```

## `! this session  integration not loaded in the shell that ran doctor`

The integration is not active in your current shell. If you have just
installed it, reload your rc file (the doctor's hint names it), or open a new
terminal:

```sh
source ~/.zshrc
```

If the line is still there in a new terminal, check the `rc file` and
`rc syntax` lines below it.

## `✗ rc file  not installed in …`

Install the integration:

```sh
git-clown doctor --install
```

If you load it with your own `eval "$(command git-clown init …)"` line,
run the doctor from a shell that has loaded it; the doctor then reports the
setup as working. See [Set it up by hand](set-up-shell-integration.md#set-it-up-by-hand).

## `! rc file  outdated block in …`

The block was written by an older git-clown. Refresh it:

```sh
git-clown doctor --install
```

## `✗ rc syntax  …`

Your shell can't parse the rc file. When that happens, the shell stops reading
the file at the broken spot, so nothing after it runs, including the
git-clown block. The detail is the shell's own error message, with the line
number.

Open the file at that line and fix it. An unclosed quote, `$(`, `if` or `{`
often shows up as an error at the very end of the file, so look further up
too. Then check again with the shell itself:

```sh
zsh -n ~/.zshrc      # or: bash -n ~/.bashrc
```

No output means the file parses.

## `✗ self-test  failed (…)`

The self-test loads the integration into a clean shell, without your rc file,
and checks that both `git-clown` and `git clown` change directory. The detail
shows the exit status and the last error line from that shell.

- If the error mentions `mktemp` or a temp file, check that `$TMPDIR` (or
  `/tmp` when it is unset) exists and is writable. The wrapper creates its
  handoff file there.
- The [doctor reference](../reference/doctor.md#self-test) lists what each
  exit status means.

`! self-test  … not found on PATH, skipped` means that shell isn't installed,
so there is nothing to test.

## The doctor is happy but `git clown` doesn't change directory

Check these in order:

1. **The clone failed.** The shell only changes directory after a successful
   clone (or when the repository already exists). Read the output above the
   prompt.
2. **The shell started before you installed the integration.** Reload the rc
   file or open a new terminal.
3. **Something else defines `git` later.** Ask the shell what `git` is:

   ```sh
   type git
   ```

   It should say that `git` is a shell function. If it is an alias, or a
   function from somewhere else, see
   [Keep your own git function](set-up-shell-integration.md#keep-your-own-git-function).
4. **You ran it from a script.** Scripts don't read your interactive rc file,
   so the functions aren't defined there, and a script can't change the
   directory of the shell that started it anyway.

## `git clown --help` says "No manual entry for git-clown"

Without the shell integration, git turns `git clown --help` into
`git help clown`, which looks for a man page. Use `git-clown --help` instead.
With the integration loaded, `git clown --help` works too.

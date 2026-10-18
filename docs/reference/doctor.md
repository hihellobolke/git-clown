# Doctor checks

`git-clown doctor [--install] [--shell <SHELL>]...` prints a report of checks
in sections, then a summary line. For what to do about a failing check, see
[Fix setup problems with doctor](../how-to/fix-setup-problems.md).

## Report format

```text
git-clown doctor

Environment
  ✓ git                git version 2.45.0
  ✓ git-clown on PATH  ~/bin/git-clown
  ✓ workspace root     ~/ws (default; set $WORKSPACE_ROOT to change)

Shell
  ✓ your shell         zsh (/bin/zsh)
  ✓ this session       integration loaded (zsh); `git clown <url>` will cd into the clone

Shell integration (zsh)
  ✓ rc file            installed in ~/.zshrc
  ✓ rc syntax          zsh parses it cleanly
  ✓ self-test          `git-clown` and `git clown` wrappers cd correctly

Everything looks good.
```

| Mark | Status | Counts as a problem |
|------|--------|---------------------|
| `✓` | OK | no |
| `!` | Warning | no |
| `✗` | Failure | yes |

A line starting with `↳` under a check is a hint on how to fix it. Paths under
your home directory are shown with `~`. Colours are turned off when stdout is
not a terminal.

The summary is `Everything looks good.`, `1 problem found.` or
`<n> problems found.`

## Shells checked

The **Shell integration** section appears once for each selected shell: the
shells given with `--shell`, or else the shell named by `$SHELL` if it is bash
or zsh. If neither applies, no such section is shown.

## Environment section

| Check | ✓ when | ! when | ✗ when |
|-------|--------|--------|--------|
| `git` | `git --version` succeeds; shows its output | | git can't be run |
| `git-clown on PATH` | An executable `git-clown` is on `PATH` and is the binary running the doctor | The one on `PATH` is a different file from the one running | None on `PATH` |
| `workspace root` | The root is a directory; shows where the value came from | It doesn't exist yet (it is created on the first clone) | It exists but is not a directory |

## Shell section

| Check | ✓ when | ! when |
|-------|--------|--------|
| `your shell` | `$SHELL` is bash or zsh | `$SHELL` is unset or another shell |
| `this session` | The doctor was started by the integration's wrapper (`GIT_CLOWN_CD_FILE` and `GIT_CLOWN_SHELL` are set) | Otherwise |

## Shell integration section

| Check | ✓ when | ! when | ✗ when |
|-------|--------|--------|--------|
| `rc file` | The rc file has the current managed block; or it has none, but the integration for this shell is loaded in this session (set up by hand) | The rc file has an outdated managed block | No managed block and not loaded in this session; or the home directory is unknown |
| `rc syntax` | `<shell> -n <rc file>` succeeds | | It fails; shows the shell's first error line |
| `self-test` | Both wrappers change directory correctly | The shell isn't on `PATH` (skipped), or the running binary can't be located | The test fails; shows the exit status and the last error line |

`rc syntax` is only shown when the rc file exists and the shell is on `PATH`.
It runs the shell without any startup files and with `BASH_ENV` removed, so
nothing in the rc file is executed.

The rc file locations are listed in
[Shell integration](shell-integration.md#rc-file-block).

### Self-test

The self-test starts `bash --norc --noprofile -c …` or `zsh -f -c …`, without
your rc files, with the directory of the running binary first on `PATH` and
`GIT_CLOWN_CD_FILE`, `GIT_CLOWN_SHELL` and `BASH_ENV` removed. The script loads
the output of `git-clown init <shell>`, then:

| Exit status | Step that failed |
|-------------|------------------|
| `90` | `cd /` |
| `91` | `git-clown __selftest-cd <tmp dir>` returned non-zero |
| `92` | After `git-clown`, `$PWD` is not the temporary directory |
| `93` | `git clown __selftest-cd <tmp dir>` returned non-zero |
| `94` | After `git clown`, `$PWD` is not the temporary directory |

## Installing

With `--install`, the doctor first installs or updates the block in the rc
file of each selected shell and prints one line per shell, such as:

```text
✓ Added the zsh integration to ~/.zshrc. Reload it with source ~/.zshrc or open a new terminal.
✓ Updated the zsh integration in ~/.zshrc. Reload it with source ~/.zshrc or open a new terminal.
✓ The zsh integration is already in ~/.zshrc; nothing changed.
```

Then it runs the checks.

## Prompt

After the report, for each selected shell whose rc file has an outdated block,
or has no block while the integration isn't loaded in this session, the
doctor:

- asks `Install the <shell> integration in <rc file>? [y/N]` (or `Update …`)
  when both stdin and stdout are terminals. It reads a single key: `y` or `Y`
  installs; any other key, including Enter and Ctrl-C, declines.
- otherwise prints `To install it, run: git-clown doctor --install --shell <shell>`.

Installing from the prompt removes the `rc file` failure from the final count.

## Exit status

`0` if no check is `✗` (after any install from the prompt), `1` otherwise.

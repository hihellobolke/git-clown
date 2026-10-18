# git-clown documentation

git-clown clones git repositories into a workspace laid out by URL
(`~/ws/github.com/example/repo`), can leave your shell inside the clone, checks
its own setup with `git-clown doctor`, and can show a repository's README in a
terminal pager.

The documentation is split into four kinds, following
[Diátaxis](https://diataxis.fr/). Pick the one that matches what you need right
now.

## Tutorials: learn by doing

Start here if you are new to git-clown.

- [Getting started](tutorials/getting-started.md): install git-clown, clone
  your first repository, set up the shell integration and read a README in the
  pager.

## How-to guides: get a specific job done

- [Install git-clown](how-to/install.md)
- [Set up the shell integration](how-to/set-up-shell-integration.md)
- [Fix setup problems with doctor](how-to/fix-setup-problems.md)
- [Change the workspace root](how-to/change-workspace-root.md)
- [Use environment files for git settings](how-to/use-environment-files.md)
- [Clone a repository with worktrees](how-to/clone-with-worktrees.md)
- [Read READMEs in the terminal](how-to/read-readmes.md)

## Reference: look up the details

- [Command reference](reference/cli.md): commands, arguments, options and exit
  statuses
- [Configuration](reference/configuration.md): environment variables and
  environment files
- [Workspace layout](reference/workspace-layout.md): how URLs map to
  directories, supported URLs, the `--bare` layout
- [Shell integration](reference/shell-integration.md): the shell functions,
  the handoff file and the rc file block
- [Doctor checks](reference/doctor.md): every check, its outcomes and the
  prompt
- [Markdown pager](reference/pager.md): keys, status bar and rendering

## Explanation: understand the design

- [The URL-shaped workspace](explanation/workspace.md): why repositories go
  where they go, and how environment files fit in
- [How the shell integration works](explanation/shell-integration.md): why a
  program can't change your directory, and what the doctor trusts
- [Bare clones and worktrees](explanation/bare-clones-and-worktrees.md): what
  `--bare` sets up and why
- [The README pager](explanation/readme-pager.md): opt-in rendering, plain
  fallback and untrusted input

## Quick start

```sh
make release                 # build and install to ~/bin
git-clown doctor --install   # set up the shell integration, then reload your shell
git clown https://github.com/example/repo.git
```

The project's [README](../README.md) has a shorter overview.
`git-clown --docs` shows that README in the terminal.

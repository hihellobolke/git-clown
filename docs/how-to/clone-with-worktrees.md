# Clone a repository with worktrees

Use `--bare` when you want several branches of one repository checked out side
by side, each in its own directory. git-clown clones the repository as a bare
repository and checks out the default branch as the first
[git worktree](https://git-scm.com/docs/git-worktree).

## Clone with `--bare`

```sh
git clown --bare https://github.com/acme/widgets.git
```

You get this layout (here the default branch is `main`):

```text
~/ws/github.com/acme/widgets/
├── .bare/                    the repository itself
├── .git                      a file pointing git at .bare
├── main/                     worktree for the default branch
└── widgets.code-workspace    VS Code workspace for the whole directory
```

With the [shell integration](set-up-shell-integration.md) loaded, your shell
ends up in `main/`.

## Add a worktree for another branch

Run `git worktree add` from the repository directory (the one that contains
`.bare`), naming the directory after the branch:

```sh
cd ~/ws/github.com/acme/widgets
git worktree add release-2.x
```

The branches of the remote are available as local branches in a bare clone, so
this checks out `release-2.x` into `release-2.x/`.

To start a new branch in its own worktree:

```sh
git worktree add -b fix-login fix-login
```

The new branch starts from the default branch.

## Update from the remote

Run `git fetch` from the repository directory or from any worktree. All
worktrees share the same repository, so one fetch is enough for all of them.

## Remove a worktree you're done with

```sh
cd ~/ws/github.com/acme/widgets
git worktree remove fix-login
git branch -d fix-login     # if you no longer need the branch either
```

## Open all worktrees in VS Code

Open the workspace file git-clown wrote:

```sh
code ~/ws/github.com/acme/widgets/widgets.code-workspace
```

It contains the whole repository directory, so worktrees you add later show up
in it too.

## Repair a half-finished setup

If a `--bare` clone stopped part of the way (for example because the network
dropped during the fetch), run the same command again:

```sh
git clown --bare https://github.com/acme/widgets.git
```

git-clown skips the steps that are already done (an existing `.bare`, an
existing default-branch worktree, an existing workspace file) and runs the
rest.

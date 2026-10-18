# Bare clones and worktrees

`git clown --bare <url>` sets a repository up differently from a plain clone:
the repository data lives in a hidden `.bare` directory, and each branch you
work on gets its own directory next to it. This page explains why that layout
is useful and how its parts fit together.

## The problem with one working copy

A plain clone has one working directory, so it has one branch checked out at a
time. Reviewing a colleague's branch, fixing something on a release branch, or
running a long test suite on `main` while you keep coding all mean stashing
your work and switching back and forth, or keeping several full clones around.

git worktrees solve this: one repository, several working directories, each on
its own branch. They share the objects and the refs, so a fetch in one is seen
by all, and a branch can't be checked out in two places by accident.

## Why a bare repository underneath

You can add worktrees to a normal clone, but then one branch is special: the
one in the clone's own directory, with the other worktrees nested inside it or
scattered elsewhere. A bare repository has no working directory of its own, so
every branch gets the same treatment, as a sibling directory:

```text
widgets/
├── .bare/        repository data
├── .git          "gitdir: ./.bare"
├── main/         worktree
└── fix-login/    worktree
```

Three small pieces make this comfortable to use:

- **The `.git` file.** It contains `gitdir: ./.bare`, which tells git that the
  `widgets/` directory belongs to the repository in `.bare`. You can run
  `git worktree add`, `git fetch` or `git branch` from `widgets/` itself,
  without `cd`-ing into a worktree or typing `--git-dir`.
- **The fetch refspec.** `git clone --bare` doesn't configure
  `remote.origin.fetch`, so a later `git fetch` wouldn't update
  remote-tracking branches like `origin/main`. git-clown sets the usual
  `+refs/heads/*:refs/remotes/origin/*` and fetches once, so the repository
  behaves like a normal clone.
- **The first worktree.** git-clown reads the default branch from the bare
  repository's `HEAD` and checks it out, so you have something to work in
  straight away, and the shell integration takes you there.

A `<repo>.code-workspace` file covers the whole directory, so VS Code shows
every worktree in one window, including ones you add later.

## Safe to run again

Every step checks whether its result already exists and skips itself if it
does. Running the same `--bare` command again finishes a setup that was
interrupted. On a complete setup it only rewrites the `.git` pointer, sets the
refspec again and fetches.

## When not to use it

The plain layout is simpler, and it's what most tools and tutorials expect.
If you only ever work on one branch of a repository, `--bare` adds a directory
level for nothing. It pays off for repositories where you regularly have more
than one branch on the go.

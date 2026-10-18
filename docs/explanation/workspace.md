# The URL-shaped workspace

git-clown exists to answer one question you'd otherwise answer by hand every
time you clone: *where does this repository go?* This page explains the answer
it gives and what follows from it.

## One place for every repository

Plain `git clone` puts a repository wherever you happen to be. Over time that
produces a home directory with `repo`, `repo-old`, `repo2` and three copies of
the same project in different places, and no way to tell which is which or
whether you have a copy at all.

git-clown removes the choice. The path comes from the URL:

```text
https://github.com/example/repo.git  →  ~/ws/github.com/example/repo
```

The host and owner are part of the path because they are part of the
repository's identity. Two projects both called `utils` from different
organisations, or a fork and its upstream, get different directories without
you having to rename anything. The tree also reads the way you think about
remotes: everything from one host is in one place, and everything from one
organisation is next to each other.

## Cloning becomes safe to repeat

Because a URL always maps to the same directory, git-clown can tell that a
repository is already there, and then does nothing except report where it is.
That turns `git clown <url>` from "make a new copy" into "make sure I have this
and take me to it". With the shell integration, it is a way to jump to any
repository you know the URL of, whether or not you cloned it before.

## Naming the clone like git does

The last directory is named the way `git clone` names it: the last part of the
URL, without `.git`. git-clown passes that name to `git clone` explicitly, so
the directory it checks for an existing clone, the directory git creates and
the directory it hands to your shell are always the same one.

This matters for hosts with nested groups, such as GitLab's
`gitlab.com/org/subgroup/repo`. The layout keeps only the host, the first path
segment and the last one, so this repository lands in `gitlab.com/org/repo`.
That keeps paths short, at a cost: two repositories with the same name in
different subgroups of one organisation map to the same directory, and the
second clone is treated as already present.

## The directory tree as configuration

Once every repository has a known place, the directories above it become
useful. Everything under `~/ws/github.com/acme/` belongs to one organisation,
so it's a natural place to put settings for that organisation: a different SSH
key, a proxy, a trace flag while debugging. That is what environment files
are for. git-clown looks for an `.envrc` or `.env` starting from the
organisation directory and walking up to the workspace root, and runs git with
what it finds.

A few choices keep this predictable:

- **Nearest file wins, alone.** Files are not merged across levels. When you
  read one file you know everything that applies, without having to check
  every directory above it.
- **The file is read, not run.** An `.envrc` is parsed for `NAME=VALUE`
  assignments; it is never executed. Cloning doesn't run code from a file in
  your workspace, and the result doesn't depend on which shell or tools you
  have. The name and the `export` syntax match direnv, so a simple direnv
  `.envrc` works for both, but direnv's functions (`use`, `layout`) do
  nothing here.
- **Only git sees it.** The variables go to the git commands git-clown runs for
  that clone. They don't leak into your shell, and they don't change how
  git-clown itself behaves.

That last point is also the limit of the feature: settings that git needs
later, every time you fetch or push, belong in the repository's git
configuration, not in an environment file.

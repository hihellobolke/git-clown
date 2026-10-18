# Workspace layout

git-clown places every repository at a path derived from its URL, under the
workspace root.

## Workspace root

`$WORKSPACE_ROOT` if set, else `~/ws`. When the default is used and `~/ws`
does not exist, it is created. See
[Configuration](configuration.md#environment-variables).

## Path of a clone

```text
<workspace root>/<host>/<owner>/<name>
```

| Part | Taken from |
|------|------------|
| `<host>` | The URL's host name, without user name or port. |
| `<owner>` | The first segment of the URL path. |
| `<name>` | The last segment of the URL path, with a trailing `/`, `/.git` or `.git` removed: the directory name `git clone` itself would choose. |

Examples:

| URL | Path under the workspace root |
|-----|-------------------------------|
| `https://github.com/example/repo.git` | `github.com/example/repo` |
| `https://github.com/example/repo` | `github.com/example/repo` |
| `git@github.com:example/repo.git` | `github.com/example/repo` |
| `ssh://git@git.example.com:2222/team/tool.git` | `git.example.com/team/tool` |
| `https://gitlab.com/org/subgroup/repo.git` | `gitlab.com/org/repo` |

A repository counts as already cloned when `<name>/.git` is a directory. Then
git-clown runs no git command and reports the existing directory.

## Supported URLs

| Form | Example |
|------|---------|
| URL with a scheme and a host | `https://host/owner/repo.git`, `ssh://user@host/owner/repo.git`, `git://host/owner/repo` |
| scp-like SSH, with a user name | `git@host:owner/repo.git` |

The path must have at least two segments (owner and repository). Not
supported:

- local paths and `file://` URLs (no host)
- scp-like URLs without a user name, such as `host:owner/repo.git`
- URLs with only one path segment, such as `git@host:repo.git`

## Bare layout

With `--bare`, the repository directory is
`<workspace root>/<host>/<owner>/<repo>`, where `<repo>` is the second segment
of the URL path without `.git`. It contains:

| Entry | Contents |
|-------|----------|
| `.bare/` | The bare repository (`git clone --bare <url> .bare`). |
| `.git` | A file containing `gitdir: ./.bare`, so git commands work from the repository directory. |
| `<branch>/` | A worktree for the default branch (the branch `HEAD` points to in the bare repository). |
| `<repo>.code-workspace` | A VS Code workspace whose only folder is the repository directory. |

Setup steps, each skipped when its result already exists:

1. `git clone --bare <url> .bare`, skipped if `.bare` exists.
2. Write the `.git` file (always rewritten).
3. Set `remote.origin.fetch` to `+refs/heads/*:refs/remotes/origin/*` (bare
   clones leave it unset) and run `git fetch`.
4. Read the default branch from `HEAD`. Fails for an empty repository.
5. `git worktree add <branch>`, skipped if `<branch>/` exists.
6. Write `<repo>.code-workspace`, skipped if it exists.

The directory reported to the shell integration, and searched for a README, is
the default-branch worktree.

For URLs with nested namespaces (`org/subgroup/repo`), the bare layout
currently uses the second path segment (`subgroup`) as `<repo>`, unlike a plain
clone, which uses the last one.

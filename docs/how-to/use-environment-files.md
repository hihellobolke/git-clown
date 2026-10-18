# Use environment files for git settings

git-clown runs git with extra environment variables taken from the nearest
`.envrc` or `.env` file in the workspace. Use this to give git different
settings for one host or one organisation, without changing your global git
configuration.

The examples below use an organisation called `acme` on GitHub. Replace the
paths with your own; `~/ws` is the [workspace root](change-workspace-root.md).

## Use a different SSH key for one organisation

Create `~/ws/github.com/acme/.envrc`:

```sh
export GIT_SSH_COMMAND="ssh -i ~/.ssh/id_ed25519_acme -o IdentitiesOnly=yes"
```

Now clones of `acme` repositories use that key:

```sh
git clown git@github.com:acme/widgets.git
```

The log names the file it used:

```text
[INFO  git_clown] Found .envrc file at: /home/you/ws/github.com/acme/.envrc
```

Clones from any other organisation are not affected.

## Use a proxy for one host

Put a file one level up, in the host directory, to cover every organisation on
that host. For example, to reach `git.example.com` through a proxy, create
`~/ws/git.example.com/.env`:

```sh
https_proxy=http://proxy.example.com:8080
```

## Trace a clone

To see what git does during a clone, add a tracing variable to the file for
that organisation, then run the clone:

```sh
export GIT_TRACE=1
```

git's trace appears with the rest of its output when the clone finishes.
Remove the line again when you are done.

## Things to know

- Only the **nearest** file is used. git-clown starts in
  `<workspace root>/<host>/<owner>` and walks up to the workspace root; the
  first directory that contains an `.envrc` or `.env` wins, and files further
  up are ignored.
- In one directory, `.envrc` is preferred over `.env`.
- Values from the file win over variables of the same name in your
  environment.
- The variables only reach the git commands git-clown runs itself: the clone
  (and, for `--bare`, the fetch and worktree setup). Later git commands you run
  in the repository don't see them. For settings that should stay with the
  repository, use `git config` instead.
- git-clown reads the file; it does not run it as a shell script. In an
  `.envrc`, `$HOME` or `~` in a value is passed to git as written. That is fine
  for `GIT_SSH_COMMAND`, whose command line git runs through a shell, but for
  other variables write out the full path, or use an `.env` file, which expands
  `$VAR` and `${VAR}`.

See [Environment files](../reference/configuration.md#environment-files) for
the exact format.

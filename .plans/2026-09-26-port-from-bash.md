# Port `wt` from bash to Rust

The bash original lives at `midgard/dotfiles/local/bin/wt`. Reads go through `gix` (`src/repo.rs`)
so completion and `prune`'s per-worktree checks don't pay for a process per question. Anything
that writes shells out to `git` (`src/git.rs`), so hooks, config and git's own refusal to remove a
dirty or locked worktree keep applying.

## Contracts other code depends on

These are the things a port must not break, because something outside this repo reads them.

- `wt switch` / `wt sw` prints the worktree path and nothing else on stdout. The zsh function in
  `dotfiles/zsh/functions/wt` does `cd "$(command wt "$@")"` and bails on a non-zero exit.
- `wt complete switch|remove` (and the `sw`/`rm` aliases) prints one branch per line;
  `dotfiles/zsh/completion.zsh` pipes it into fzf. `remove` leaves out the main worktree.
- A linked worktree lives at `<parent of main>/<main basename>-<branch>`. nvim's
  `utils/worktree/git.lua` strips that prefix to recover the name `wt sw` takes.

One deliberate change: all log output goes to stderr. The script printed `log`/`warn` to stdout
and `switch -c` redirected `new` to keep stdout clean; with everything on stderr that redirect goes
away.

## Commands

**new `<branch>`.** Derive the path from the main worktree, even when run from a linked one, so it
never produces `repo-a-b`. Refuse if the branch is already checked out in some worktree, or if the
target directory exists. Then pick the branch source: existing local branch, else create it
tracking `origin/<branch>`, else create it from HEAD. Finally `git worktree add <path> <branch>`.

**switch `[-c|--create] <branch>` (alias `sw`).** `--create` runs `new` first. `main`, `master` and
whatever the main worktree has checked out resolve to the main worktree; anything else to the
sibling path. Error if that directory doesn't exist. Note it checks the *directory*, not the
worktree list — a worktree created outside `wt` under another name isn't reachable by `switch`.

**remove `<branch>` (alias `rm`).** Find the worktree with that branch checked out and
`git worktree remove` it. Never `--force`.

**list.** Same output as `git worktree list`: path, short HEAD, `[branch]` or `(detached HEAD)`,
with paths padded to align.

**prune.** Default branch is `origin/HEAD`'s target, falling back to local `main`, then `master`,
else fail. For every linked worktree with a branch: if the branch is an ancestor of the default
branch *and* its reflog has more than one entry, remove it. An ancestor with a single-entry (or
missing) reflog is an untouched fresh branch and is kept with a warning. Exit non-zero if any
removal was refused.

**complete `<subcommand>`.** Hidden. See contracts above.

## Where each piece lands

| Bash                                      | Rust                                  | Via  |
|-------------------------------------------|---------------------------------------|------|
| `git worktree list --porcelain` parsing   | `Repo::worktrees`, `main_worktree`    | gix  |
| `git worktree list` (for `list`)          | `Repo::worktrees` + formatting        | gix  |
| `git symbolic-ref --short HEAD` in main   | `Repo::main_branch`                   | gix  |
| `git show-ref --verify refs/heads/…`      | `Repo::local_branch_exists`           | gix  |
| `git show-ref --verify refs/remotes/…`    | `Repo::remote_branch_exists`          | gix  |
| `git symbolic-ref refs/remotes/origin/HEAD` | `Repo::default_branch`              | gix  |
| `git merge-base --is-ancestor`            | `Repo::is_ancestor`                   | gix  |
| `git reflog show … \| wc -l`              | `Repo::branch_has_own_commits`        | gix  |
| `git branch <b> [origin/<b>]`             | `git::branch_create`                  | git  |
| `git worktree add`                        | `git::worktree_add`                   | git  |
| `git worktree remove`                     | `git::worktree_remove`                | git  |

## Order of work

Each step test-first against throwaway repos built with the real `git` (`tempfile`, driving the
binary via `CARGO_BIN_EXE_wt`).

1. `Repo::discover`, `worktrees`, then `list` and `complete` — read-only, and it settles the
   worktree model everything else uses.
2. `switch` without `-c`.
3. `git` module, then `new` and `switch --create`.
4. `remove`.
5. `prune`, with tests for the merged, unmerged and untouched-fresh-branch cases.
6. Point the dotfiles at the binary and retire the script.

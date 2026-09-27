# wt

`wt` keeps one git worktree per branch, each checked out as a sibling of your main checkout:

```
~/code/proj            main
~/code/proj-login      login
~/code/proj-fix-typo   fix-typo
```

Switching branches becomes changing directory, so a half-finished change, a running build or a
review never has to be stashed to make room for something else.

## Install

```sh
cargo install --path .
```

Then load the shell integration in your `~/.zshrc`:

```zsh
eval "$(wt init zsh)"
```

A program can't change its parent shell's directory, so `wt switch` prints the worktree's path and
the `wt` function this defines `cd`s into it. It also wires up fzf's `**<TAB>` completion for
`wt switch`, `wt new` and `wt remove`. Only zsh is supported.

## Usage

```sh
wt switch login         # cd into login's worktree (alias: wt sw)
wt switch -c spike      # a branch that doesn't exist yet: create it from HEAD, then cd
wt new login            # create the worktree without switching to it
wt list                 # the same output as git worktree list
wt remove login         # remove login's worktree (alias: wt rm); the branch stays
wt prune                # remove every worktree whose branch is merged
```

`wt switch main`, or whatever your default branch is called, always takes you back to the main
checkout.

### Reviewing someone else's branch

Fetch, then switch to the branch by name:

```sh
git fetch
wt sw fix-typo
```

If `fix-typo` has no worktree yet, `wt` creates one: from your local branch if you have it,
otherwise from a new local branch tracking `origin/fix-typo`. `wt sw **<TAB>` lists your worktrees
followed by branches you could open one for, marked `local` or `origin`. `wt` never fetches on its
own, so completion stays instant and works offline.

### What prune removes

The default branch is `origin/HEAD`'s target, or whatever the main checkout has on it. `prune`
removes a linked worktree when its branch is merged into the default branch and has commits of its
own. A branch that was created and never committed to is trivially "merged", so it is kept with a
warning. Removal goes through `git worktree remove` without `--force`, so a worktree with
uncommitted or untracked changes is refused and left alone, and `prune` exits non-zero. The
worktree holding the default branch is never removed.

## Development

`CLAUDE.md` covers the design and conventions. In short: `cargo test` runs integration tests that
drive the built binary against throwaway git repositories, and `cargo clippy --all-targets` and
`cargo fmt --check` must both be clean.

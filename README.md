# wt

`wt` keeps one git worktree per branch, each checked out beside your main checkout:

```
~/code/proj            main
~/code/proj-login      login
~/code/proj-fix-typo   fix-typo
```

Switching branches becomes changing directory, so a half-finished change, a running build or a
review never has to be stashed to make room for something else.

## Install

On x86_64 Linux, download the static binary from the latest GitHub release and put it on your
`PATH`. It needs `git` at runtime.

```sh
curl -fsSLO https://github.com/alksl/wt/releases/latest/download/wt-x86_64-unknown-linux-musl.tar.gz
tar xzf wt-x86_64-unknown-linux-musl.tar.gz
install -m755 wt-x86_64-unknown-linux-musl/wt ~/.local/bin/
```

The release notes show how to check it against `SHA256SUMS` and its signature. Elsewhere, build it
with Cargo:

```sh
cargo install --git https://github.com/alksl/wt
```

Then load the shell integration from your `~/.zshrc`:

```zsh
eval "$(wt init zsh)"
```

A program can't change its parent shell's directory, so `wt switch` prints the worktree's path and
the `wt` function this defines `cd`s into it. It also wires up fzf's `**<TAB>` completion for
`wt switch`, `wt new`, `wt exec` and `wt remove`. Only zsh is supported.

## Usage

```sh
wt switch login         # cd into login's worktree (alias: wt sw)
wt switch -c spike      # a branch that doesn't exist yet: create it from HEAD, then cd
wt new login            # create the worktree without switching to it
wt exec login -- make   # run a command in login's worktree, staying where you are
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
followed by the branches you could open one for, marked `local` or `origin`. `wt` never fetches on
its own, so completion stays instant and works offline.

### What prune removes

`prune` removes a linked worktree when its branch is merged into the default branch and has commits
of its own. The default branch is `origin/HEAD`'s target, or whatever the main checkout has on it,
and its own worktree is never removed. A branch that was created and never committed to counts as
merged, so it is kept, with a warning. Removal goes through `git worktree remove` without `--force`:
a worktree with uncommitted or untracked changes is left alone, and `prune` exits non-zero.

## Development

`cargo test` runs integration tests that drive the built binary against throwaway git
repositories, and `cargo clippy --all-targets` and `cargo fmt --check` must both be clean.
`CLAUDE.md` describes the design and the contracts the shell integration relies on. Releases are cut
with `make release`; [RELEASING.md](RELEASING.md) explains it.

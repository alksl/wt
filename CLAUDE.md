# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`wt` manages git worktrees that live beside the main checkout as `<repo>-<branch>`, one per
branch, and ships the zsh integration that lets `wt switch` change directory. `README.md` describes
it from the user's side.

## Commands

Rust comes from mise (`.tool-versions`, `rust latest`), so the shell needs mise activated.

```sh
cargo build
cargo clippy --all-targets
cargo fmt --check
cargo test --test switch                               # one command's tests
cargo test --test switch prints_only_the_worktree_path # one test
```

## Design

Reads go through `gix`, so shell completion and `prune`'s per-worktree checks don't pay for a git
process per question. Anything that changes the repository shells out to the `git` binary, so hooks,
user config and git's own refusal to remove a dirty or locked worktree keep applying. Don't move a
mutation into `gix`, and never pass `--force` to `git worktree remove`.

The CLI is clap derive in `src/main.rs`; each subcommand's help text is the `///` comment on its
variant. Each command lives in its own file under `src/cmd/`.

## Contracts with callers outside this repo

- `wt switch` prints the worktree path and nothing else on stdout; the zsh function cds to it.
  All logging therefore goes to stderr.
- `wt complete switch|new|remove|exec` (hidden) prints one branch per line for the fzf completion.
  `remove` lists the linked worktrees' branches. `new`, which also serves `wt sw -c`, lists the
  branches with no worktree, each followed by a tab and `local` or `origin` (origin only when
  there's no local branch of that name). `exec` lists every worktree's branch, main first, and
  `switch` lists those, then `new`'s lines. The completion's `_post` function cuts at the tab.
- The zsh function and fzf completion come from `wt init zsh` (`src/cmd/init.zsh`), which the
  user's zshrc `eval`s.
- nvim's `utils/worktree/git.lua` strips the `<repo>-` prefix off a worktree directory to get the
  name `wt switch` takes, so the naming scheme is load-bearing.

## Tests

Tests are integration tests that run the built binary (`CARGO_BIN_EXE_wt`) against real throwaway
repositories from `TestRepo` in `tests/common/mod.rs`. The repo sits at `<tmp>/proj` so worktrees
land beside it as `<tmp>/proj-<branch>`. Git runs with no system or global config and `LC_ALL=C`,
so tests may match git's English messages. Build a test's starting state with plain `git` through
the fixture rather than with `wt`, so one command's tests don't depend on another command working.

`tests/acceptance.rs` is the exception: whole workflows (create, switch, list, remove, prune) run
through `wt` itself on `TestRepo::toy()`, a small project with real files and an origin.

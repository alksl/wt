# Releasing wt

A release is a signed tag `vX.Y.Z` on `main` and a GitHub release for it carrying a static x86_64
Linux binary, `wt-x86_64-unknown-linux-musl.tar.gz`, with `SHA256SUMS` and `SHA256SUMS.asc`, a
detached signature of the checksums by the same key that signs the tag. Users verify against
`0F7781B7A2AB98DFB83F01B5970E4326B217EC4B`, which keys.openpgp.org serves.

## Cutting a release

Bump `version` in `Cargo.toml` in one commit on `main`, then run `cargo update --workspace` so
`Cargo.lock` agrees. Push that commit, then run `make release`. It tags `HEAD` as the version in
`Cargo.toml`, builds and signs the tarball into `dist/`, signs the tag, pushes `main` and the tag to
`origin` and `github`, and creates the GitHub release with the commit subjects since the previous
tag as its notes and `dist/*` attached. It commits nothing and never force-pushes. The steps are the
`release` recipe in `Makefile`, one command each; the checks, the signing key, the tarball and the
notes are scripts in `scripts/`.

It signs with this machine's key: `git config user.signingkey`, or else makepkg's `GPGKEY`, which
midgard sets per host (midgard's `docs/gpg-signing.md` covers giving a new machine its subkey). It
also needs the musl target (`rustup target add x86_64-unknown-linux-musl`), `gh auth login` done
and a `github` remote (`git remote add github git@github.com:alksl/wt.git`).

Nothing it publishes can be overwritten, so before changing anything it stops when:

- the version isn't `x.y.z`, or isn't newer than the latest `v*` tag
- the tree is dirty, `main` isn't checked out, or `main` differs from origin's
- the tag already exists locally or on either remote, or a GitHub release for it exists
- gh isn't logged in, or this machine has no signing key or lacks its secret

## Testing it without publishing

`scripts/build-dist` builds and signs exactly what the release attaches, and nothing it does leaves
the machine:

```sh
scripts/build-dist "$(scripts/signing-key)"
(cd dist && sha256sum -c SHA256SUMS && gpg --verify SHA256SUMS.asc SHA256SUMS)
scripts/release-notes v0.2.0
```

## Finishing a release by hand

These are the steps `make release` runs after its checks and the build, for when it stops partway.
The `!` after the subkey makes GnuPG use exactly that subkey, not the newest one, whose secret may
live on another machine.

```sh
git tag -s -u '<your subkey>!' -m v0.2.0 v0.2.0
git push --atomic origin main refs/tags/v0.2.0
git push --atomic github main refs/tags/v0.2.0
scripts/release-notes v0.2.0 |
  gh release create v0.2.0 --repo alksl/wt --verify-tag --title v0.2.0 --notes-file - dist/*
```

If the release exists but its files didn't upload, `gh release upload v0.2.0 dist/* --repo alksl/wt`
adds them.

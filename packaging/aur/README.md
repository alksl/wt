# Releasing wt

A release is a signed tag `vX.Y.Z` on `main`. The AUR package `wt-switcher` builds from that tag on
`https://github.com/alksl/wt`, and `makepkg` refuses to build unless the tag is signed by the key in
`validpgpkeys` or one of its valid subkeys. AUR helpers fetch that key by fingerprint from
keys.openpgp.org; by hand it is:

```sh
gpg --keyserver hkps://keys.openpgp.org --recv-keys 0F7781B7A2AB98DFB83F01B5970E4326B217EC4B
```

The package isn't called `wt` because the official repos already have one, the Wt C++ toolkit. The
binary it installs is still `wt`.

## Cutting a release

Bump the version in one commit on `main`: `version` in `Cargo.toml` (then
`cargo update --workspace`), `pkgver` in `PKGBUILD` with `pkgrel=1`, and `.SRCINFO`, which the AUR
reads instead of running the PKGBUILD:

```sh
(cd packaging/aur && makepkg --printsrcinfo > .SRCINFO)
```

Push that commit, then run `make release`. It tags `HEAD` as the version in `Cargo.toml`, signs the
tag, pushes `main` and the tag to `origin` and `github`, creates the GitHub release with the commit
subjects since the previous tag as its notes, and pushes the PKGBUILD to the AUR from a clone in
`~/.cache/wt/aur`. It commits nothing and never force-pushes. The steps are the `release` recipe
in `Makefile`, one command each; the checks, the signing key, the notes and the AUR push are
scripts in `scripts/`.

It signs with this machine's key: `git config user.signingkey`, or else makepkg's `GPGKEY`, which
midgard sets per host (midgard's `docs/gpg-signing.md` covers giving a new machine its subkey). It
also needs `gh auth login` done, a `github` remote
(`git remote add github git@github.com:alksl/wt.git`) and your SSH key on your AUR account.

Nothing it publishes can be overwritten, so before changing anything it stops when:

- the version isn't `x.y.z`, or isn't newer than both the latest `v*` tag and the AUR's version
- the tree is dirty, `main` isn't checked out, or `main` differs from origin's
- `pkgver` isn't the Cargo version, `pkgrel` isn't 1, or `.SRCINFO` is stale
- the tag already exists locally or on either remote, or a GitHub release for it exists
- gh isn't logged in, or this machine has no signing key or lacks its secret

## Changing only the packaging

When the packaging changes but the source is the same tag (a dependency, an install path), bump
`pkgrel` rather than `pkgver` and regenerate `.SRCINFO`. There is no new tag, so `make release`
refuses this; push the two files to the AUR yourself:

```sh
git -C ~/.cache/wt/aur pull --ff-only
cp packaging/aur/{PKGBUILD,.SRCINFO} ~/.cache/wt/aur/
git -C ~/.cache/wt/aur commit -am 'Update to 0.2.0-2'
git -C ~/.cache/wt/aur push
```

## Finishing a release by hand

These are the steps `make release` runs after its checks, for when it stops partway. The `!` after
the subkey makes GnuPG use exactly that subkey, not the newest one, whose secret may live on
another machine.

```sh
git tag -s -u '<your subkey>!' -m v0.2.0 v0.2.0
git push --atomic origin main refs/tags/v0.2.0
git push --atomic github main refs/tags/v0.2.0
scripts/release-notes v0.2.0 |
  gh release create v0.2.0 --repo alksl/wt --verify-tag --title v0.2.0 --notes-file -
scripts/publish-aur 0.2.0   # clones or pulls ~/.cache/wt/aur, commits, pushes to master
```

The AUR only accepts a branch named `master`, and the first push creates the package. If you have
`namcap` installed, `namcap PKGBUILD` and `namcap wt-switcher-*.pkg.tar.zst` are worth a run
before pushing.

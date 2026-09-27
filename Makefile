SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.ONESHELL:

GITHUB_REPO := alksl/wt
AUR_REMOTE := ssh://aur@aur.archlinux.org/wt-switcher.git
AUR_DIR := $${XDG_CACHE_HOME:-$$HOME/.cache}/wt/aur

.PHONY: release

# make release: tags HEAD as the version in Cargo.toml, signs and pushes the tag, creates the GitHub
# release and publishes the PKGBUILD to the AUR. It commits nothing; packaging/aur/README.md says
# how to bump the version first. Needs gh, jq, gpg, makepkg and vercmp.
release:
	@die() { echo "release: $$*" >&2; exit 1; }
	version=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
	tag=v$$version
	read -r pkgver pkgrel <<< "$$(source packaging/aur/PKGBUILD && echo "$$pkgver $$pkgrel")"
	latest=$$(git tag -l 'v*' --sort=-v:refname | head -1)
	aur_version=$$(curl -fsSg "https://aur.archlinux.org/rpc/v5/info?arg[]=wt-switcher" |
	  jq -r '.results[0].Version // empty')
	# This machine's signing key: git's, else makepkg's GPGKEY (the user file overrides /etc).
	key=$$(git config user.signingkey || true)
	[[ -n $$key ]] || key=$$(
	  set +u
	  for f in /etc/makepkg.conf "$${XDG_CONFIG_HOME:-$$HOME/.config}/pacman/makepkg.conf"; do
	    if [[ -r $$f ]]; then source "$$f"; fi
	  done
	  echo "$${GPGKEY:-}"
	)

	# Guards: nothing below this may overwrite or go backwards from a published release.
	[[ $$version =~ ^[0-9]+\.[0-9]+\.[0-9]+$$ ]] ||
	  die "Cargo.toml version must be x.y.z, got '$$version'"
	[[ -z $$(git status --porcelain) ]] || die "working tree is not clean"
	[[ $$(git symbolic-ref --short -q HEAD) == main ]] || die "not on main"
	[[ $$(git rev-parse HEAD) == "$$(git ls-remote origin refs/heads/main | cut -f1)" ]] ||
	  die "main is not the same commit as origin's main"
	[[ $$pkgver == "$$version" ]] ||
	  die "PKGBUILD pkgver is $$pkgver, Cargo.toml says $$version"
	[[ $$pkgrel == 1 ]] || die "PKGBUILD pkgrel is $$pkgrel; a new version starts at 1"
	diff -q <(cd packaging/aur && makepkg --printsrcinfo) packaging/aur/.SRCINFO > /dev/null ||
	  die ".SRCINFO is stale; run makepkg --printsrcinfo > .SRCINFO in packaging/aur"
	[[ -z $$latest ]] || (($$(vercmp "$$version" "$${latest#v}") > 0)) ||
	  die "$$tag is not newer than $$latest"
	! git rev-parse -q --verify "refs/tags/$$tag" > /dev/null || die "tag $$tag exists locally"
	for remote in origin github; do
	  [[ -z $$(git ls-remote --tags $$remote "refs/tags/$$tag") ]] ||
	    die "tag $$tag exists on $$remote"
	done
	gh auth status > /dev/null 2>&1 || die "gh is not logged in; run gh auth login"
	! gh release view "$$tag" --repo "$(GITHUB_REPO)" > /dev/null 2>&1 ||
	  die "GitHub release $$tag exists"
	[[ -z $$aur_version ]] || (($$(vercmp "$$version-1" "$$aur_version") > 0)) ||
	  die "$$version-1 is not newer than $$aur_version on the AUR"
	[[ -n $$key ]] || die "no signing key; set git config --global user.signingkey" \
	  "'<subkey>!' or gpg_signing_subkey in this host's midgard host_vars"
	gpg --list-secret-keys "$$key" > /dev/null 2>&1 || die "no secret key for signing key $$key"

	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test

	notes='First release.'
	[[ -z $$latest ]] || notes=$$(git log --format='- %s' "$$latest..HEAD")
	notes+=$$'\n\nInstall from the AUR: https://aur.archlinux.org/packages/wt-switcher'

	git tag -s -u "$$key" -m "$$tag" "$$tag"
	git tag -v "$$tag"
	git push --atomic origin main "refs/tags/$$tag"
	git push --atomic github main "refs/tags/$$tag"
	gh release create "$$tag" --repo "$(GITHUB_REPO)" --verify-tag --title "$$tag" \
	  --notes "$$notes"

	if [[ -d "$(AUR_DIR)" ]]; then
	  git -C "$(AUR_DIR)" pull --ff-only
	else
	  git clone "$(AUR_REMOTE)" "$(AUR_DIR)"
	fi
	cp packaging/aur/PKGBUILD packaging/aur/.SRCINFO "$(AUR_DIR)/"
	git -C "$(AUR_DIR)" add PKGBUILD .SRCINFO
	git -C "$(AUR_DIR)" commit -m "Update to $$version-1"
	git -C "$(AUR_DIR)" push -u origin HEAD:master

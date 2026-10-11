# make release: tags HEAD as the version in Cargo.toml and publishes it to GitHub and the AUR.
# packaging/aur/README.md covers bumping the version first and finishing a release by hand.
SHELL := bash
.SHELLFLAGS := -o pipefail -c

VERSION = $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
TAG = v$(VERSION)
KEY = $(shell scripts/signing-key)

.PHONY: release
release:
	scripts/release-guards '$(VERSION)' '$(KEY)'
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test
	git tag -s -u '$(KEY)' -m $(TAG) $(TAG)
	git tag -v $(TAG)
	git push --atomic origin main refs/tags/$(TAG)
	git push --atomic github main refs/tags/$(TAG)
	scripts/release-notes $(TAG) | \
	  gh release create $(TAG) --repo alksl/wt --verify-tag --title $(TAG) --notes-file -
	scripts/publish-aur $(VERSION)

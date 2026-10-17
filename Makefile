fmt:
	cargo fmt
test:
	cargo test --features "image-encoder"
lint:
	cargo clippy --features=image-encoder --all-targets --all -- --deny=warnings
udeps:
	cargo +nightly udeps
bench:
	cargo bench --features "image-encoder"
changelog:
	git cliff --config cliff.toml --output CHANGELOG.md

# Versions. `make version-patch`, `make version-minor` or `make version-major`
# set the next version in Cargo.toml and Cargo.lock (and, when the major one
# changes, in the install lines of the README and the guide), and write the
# changelog with the commits since the last tag under that version. Nothing
# is committed or tagged: look the changes over, then commit and tag them as
# the last lines of the output say.
VERSION := $(shell sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -n 1)
MAJOR := $(word 1,$(subst ., ,$(VERSION)))
MINOR := $(word 2,$(subst ., ,$(VERSION)))
PATCH := $(word 3,$(subst ., ,$(VERSION)))
NEXT_MAJOR = $(word 1,$(subst ., ,$(NEXT)))
INSTALL_DOCS := README.md README-zh.md docs/guide.md docs/guide-zh.md

version-patch:
	@$(MAKE) --no-print-directory version NEXT=$(MAJOR).$(MINOR).$(shell echo $$(($(PATCH) + 1)))
version-minor:
	@$(MAKE) --no-print-directory version NEXT=$(MAJOR).$(shell echo $$(($(MINOR) + 1))).0
version-major:
	@$(MAKE) --no-print-directory version NEXT=$(shell echo $$(($(MAJOR) + 1))).0.0

version:
	@test -n "$(NEXT)" || { echo "Use version-patch, version-minor or version-major."; exit 1; }
	@test -n "$(PATCH)" || { echo "The version in Cargo.toml is not x.y.z: $(VERSION)"; exit 1; }
	@git diff --quiet HEAD -- || { echo "Commit or stash the changes first: the changelog is made of the commits."; exit 1; }
	@command -v git-cliff >/dev/null || { echo "git-cliff is needed: cargo install git-cliff"; exit 1; }
	@! git rev-parse -q --verify "refs/tags/v$(NEXT)" >/dev/null || { echo "v$(NEXT) is a tag already."; exit 1; }
	@perl -pi -e 's/^version = "\Q$(VERSION)\E"/version = "$(NEXT)"/' Cargo.toml
	@cargo metadata --format-version 1 --offline >/dev/null
	@if [ "$(NEXT_MAJOR)" != "$(MAJOR)" ]; then \
		perl -pi -e 's/charts-rs = "$(MAJOR)"/charts-rs = "$(NEXT_MAJOR)"/g; s/charts-rs = \{ version = "$(MAJOR)"/charts-rs = { version = "$(NEXT_MAJOR)"/g' $(INSTALL_DOCS); \
	fi
	@git cliff --config cliff.toml --tag v$(NEXT) --output CHANGELOG.md
	@echo "$(VERSION) -> $(NEXT). Look the changes over (git diff), then:"
	@echo "  git commit -am 'chore: version $(NEXT)'"
	@echo "  git tag v$(NEXT)"

.PHONY: fmt test lint udeps bench changelog version version-patch version-minor version-major

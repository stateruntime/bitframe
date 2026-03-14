# Mirrors .github/workflows/ci.yml — run `just check` before pushing.

default: check

# Run the full CI pipeline locally
check: fmt-check clippy test test-minimal doc deny

# Format all code
fmt:
    cargo fmt --all

# Check formatting (CI mode)
fmt-check:
    cargo fmt --all --check

# Lint with clippy
clippy:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# Run all tests
test:
    cargo test --workspace --all-features

# Run tests without default features (no_std compatibility)
test-minimal:
    cargo test --workspace --no-default-features

# Build docs (warnings are errors)
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features

# Run cargo-deny
deny:
    cargo deny check

# Quick compile check
dev:
    cargo check --workspace --all-features

# Sync VERSION into all Cargo.toml files
release:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(cat VERSION | tr -d '[:space:]')
    for f in crates/bitframe/Cargo.toml crates/bitframe-derive/Cargo.toml; do
        sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" "$f" && rm "$f.bak"
    done
    echo "Synced version $VERSION into workspace Cargo.toml files"

# Test all feature combinations (requires cargo-hack)
test-features:
    cargo hack test --feature-powerset --no-dev-deps

# Check semver compatibility (requires cargo-semver-checks)
semver:
    cargo semver-checks check-release

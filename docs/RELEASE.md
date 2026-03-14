# Release Process

How to publish a new version of bitframe.

## Steps

### 1. Update version

```bash
echo "0.2.0" > VERSION
just release          # syncs VERSION into Cargo.toml
```

### 2. Update CHANGELOG.md

Rename `[Unreleased]` to the new version with today's date. Add a fresh `[Unreleased]` section:

```markdown
## [Unreleased]

## [0.2.0] - 2026-04-01

### Added
- ...
```

Update the comparison links at the bottom:

```markdown
[Unreleased]: https://github.com/stateruntime/bitframe/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/stateruntime/bitframe/compare/v0.1.0...v0.2.0
```

### 3. Pre-release checks

```bash
just check              # fmt, clippy, test, test-minimal, doc, deny
cargo publish --dry-run  # verify packaging
```

### 4. Commit and tag

```bash
git add VERSION Cargo.toml CHANGELOG.md
git commit -m "chore: release v0.2.0"
git tag v0.2.0
git push origin main v0.2.0
```

### 5. Automated publish

The `v*` tag triggers `.github/workflows/release.yml` which:

1. Validates that the tag matches the `VERSION` file
2. Runs the full test suite (fmt, clippy, test)
3. Publishes to crates.io (skips gracefully if already published)
4. Creates a GitHub Release with notes extracted from `CHANGELOG.md`
5. Tags containing `-` (e.g., `v1.0.0-rc.1`) are marked as prereleases

## Multi-Crate Publishing (future)

When the workspace splits into `bitframe-core`, `bitframe-derive`, and `bitframe`:

1. Publish `bitframe-core` first
2. Wait 30 seconds for crates.io index update
3. Publish `bitframe-derive`
4. Wait 30 seconds
5. Publish `bitframe`

The release workflow has commented-out steps ready for this.

## Semver Rules

| Change | Bump |
|--------|------|
| Breaking API change | Major |
| New feature, backwards compatible | Minor |
| Bug fix, no API change | Patch |
| MSRV bump | Minor minimum |
| New optional feature flag | Minor |
| Wire semantics change | Major (after v1.0) |

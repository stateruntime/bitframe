# Contributing

Thanks for your interest in contributing to bitframe!

## Current Status

The repository is at **v0.1.0 — parsing is implemented.** Zero-copy views, bit-sized types, and proc-macros are working. Writers and mutable views are planned for future releases. Contributions to tests, protocol examples, documentation, and new features are welcome.

## Development

### Prerequisites

- Rust stable (see `rust-toolchain.toml`)
- [just](https://github.com/casey/just) (optional, but recommended)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) (for dependency audits)

### Run the full CI pipeline locally

```bash
just check
```

This runs (in order): `fmt-check`, `clippy`, `test`, `test-minimal`, `doc`, `deny`.

### Individual commands

```bash
cargo fmt --all --check                                   # formatting
cargo clippy --all-features --all-targets -- -D warnings  # linting
cargo test --all-features                                 # all tests
cargo test --no-default-features                          # no_std compatibility
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features  # docs
cargo deny check                                          # dependency audit
```

### Code rules

- `unsafe_code` is **forbidden** — no exceptions
- Clippy pedantic + nursery are enabled as warnings
- `unwrap_used`, `expect_used`, `panic`, `todo` are denied
- Tests use the [behave](https://crates.io/crates/behave) BDD framework
- MSRV: Rust 1.75

### Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

| Type | When |
|------|------|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `docs:` | Documentation only |
| `refactor:` | Code change that neither fixes nor adds |
| `test:` | Adding or updating tests |
| `chore:` | Build, CI, tooling changes |

Examples:
- `feat: add u1..u63 bit-sized types`
- `fix: correct byte-boundary detection for u11 fields`
- `docs: update CCSDS golden test vectors`
- `chore: bump MSRV to 1.76`

### Versioning

- The `VERSION` file is the single source of truth
- Run `just release` to sync it into `Cargo.toml`
- Every PR with user-facing changes should update `CHANGELOG.md`

### Pull requests

1. Fork and create a feature branch
2. Make your changes
3. Run `just check` (or the individual commands above)
4. Update `CHANGELOG.md` under `[Unreleased]`
5. Open a PR against `main`

CI will verify formatting, linting, tests (both `--all-features` and `--no-default-features`), docs, and that `VERSION` matches `Cargo.toml`.

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

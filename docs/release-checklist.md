# Release Checklist

Use this checklist for every release.

## vX.Y.Z release steps

1. Confirm CI is green on the release branch.
2. Bump `version` in `Cargo.toml`.
3. Update `CHANGELOG.md` with the release date and summary.
4. Run local verification:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all-features`
5. Confirm README install and run steps still work.
6. Create and push a tag using format `vX.Y.Z`.
7. Create GitHub release notes summarizing the changes.
